use chrono::Utc;
use ed25519_dalek::{Signature, Verifier};
use localforge_core::{LocalForgeError, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Seek, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSnapshotAssertion {
    pub id: String,
    pub name: String,
    pub path: String,
    pub volume_uuid: String,
    pub document_root: String,
    pub php_version: String,
    pub local_domain: String,
    pub assigned_loopback_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationApprovalPayload {
    pub token_id: String,
    pub database_path: String,
    pub activation_snapshot_sha256: String,
    pub scope: String,
    pub expected_projects: Vec<ProjectSnapshotAssertion>,
    pub issued_at: i64,
    pub expires_at: i64,
}

impl MigrationApprovalPayload {
    /// Produces a deterministic canonical byte sequence for asymmetric Ed25519 signing and verification.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>> {
        // Sort projects by ID for deterministic byte layout
        let mut sorted_projects = self.expected_projects.clone();
        sorted_projects.sort_by(|a, b| a.id.cmp(&b.id));

        let canonical_struct = MigrationApprovalPayload {
            token_id: self.token_id.clone(),
            database_path: self.database_path.clone(),
            activation_snapshot_sha256: self.activation_snapshot_sha256.to_lowercase(),
            scope: self.scope.clone(),
            expected_projects: sorted_projects,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
        };

        serde_json::to_vec(&canonical_struct).map_err(|e| {
            LocalForgeError::Serialization(format!("Failed to serialize canonical payload: {e}"))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationApprovalToken {
    pub payload: MigrationApprovalPayload,
    pub signature: String, // 64-byte Ed25519 signature in hex format
}

impl MigrationApprovalToken {
    /// Independent issuance helper used by the offline Product Owner tooling and verification tests.
    /// The runtime executor does NOT have access to a signing key and can only verify.
    pub fn issue_with_signing_key(
        signing_key: &ed25519_dalek::SigningKey,
        payload: MigrationApprovalPayload,
    ) -> Result<Self> {
        use ed25519_dalek::Signer;
        let canonical_bytes = payload.to_canonical_bytes()?;
        let signature = signing_key.sign(&canonical_bytes);
        Ok(Self {
            payload,
            signature: hex::encode(signature.to_bytes()),
        })
    }

    /// Serializes token to formatted JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| LocalForgeError::Serialization(e.to_string()))
    }
}

pub mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        let mut s = String::with_capacity(data.as_ref().len() * 2);
        for byte in data.as_ref() {
            s.push_str(&format!("{:02x}", byte));
        }
        s
    }

    pub fn decode(hex_str: &str) -> Option<Vec<u8>> {
        if !hex_str.len().is_multiple_of(2) {
            return None;
        }
        (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).ok())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEvidence {
    pub token_id: String,
    pub scope: String,
    pub activation_snapshot_sha256: String,
    pub consumed_at: String,
    pub signature_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumedRecord {
    pub token_id: String,
    pub consumed_at: String,
    pub scope: String,
    pub activation_snapshot_sha256: String,
}

/// RAII lock guard to ensure exclusive POSIX flock is always released upon function exit.
#[cfg(unix)]
struct FlockGuard(std::os::unix::io::RawFd);

#[cfg(unix)]
impl Drop for FlockGuard {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.0, libc::LOCK_UN);
        }
    }
}

/// Verifies and atomically consumes a Product Owner approval token using a verifier-only
/// asymmetric Ed25519 public key architecture and an OS-level flock-protected replay ledger.
///
/// Security guarantees:
/// 1. Asymmetric separation: The executor only holds the public key; automated agents cannot self-approve.
/// 2. Cryptographic binding: Binds approval to the exact DB path, signed snapshot SHA-256, scope, and rich project assertions.
/// 3. Cross-process atomic replay protection: Exclusive `flock` critical section covering read, validation, append, and durable `sync_all()`.
/// 4. Fail-closed ledger safety: Rejects symlinks, verifies file ownership, enforces 0600 mode, and fails closed on corrupted ledger lines.
/// 5. Durability: Consumed token record is written and synced to disk before migration is permitted to begin.
pub fn verify_and_consume_approval(
    token_str: &str,
    expected_db_path: &Path,
    expected_snapshot_sha256: &str,
    expected_scope: &str,
    expected_projects: &[ProjectSnapshotAssertion],
    consumed_ledger_path: &Path,
    trust_anchor_path: &Path,
) -> Result<ApprovalEvidence> {
    // 1. Resolve and verify Product Owner trust anchor (never accept arbitrary environment keys)
    let (verifying_key, _anchor) =
        crate::trust_anchor::load_verified_trust_anchor(trust_anchor_path)?;

    // 2. Parse token from JSON or Base64
    let trimmed = token_str.trim();
    let token: MigrationApprovalToken = if trimmed.starts_with('{') {
        serde_json::from_str(trimmed).map_err(|e| {
            LocalForgeError::Validation(format!("Invalid approval token JSON structure: {e}"))
        })?
    } else {
        // Attempt Base64 decode
        let stripped = trimmed.replace(['\n', '\r', ' '], "");
        let Ok(bytes) = base64_decode(&stripped) else {
            return Err(LocalForgeError::Validation(
                "Invalid approval token format: must be valid JSON or Base64".into(),
            ));
        };
        serde_json::from_slice(&bytes).map_err(|e| {
            LocalForgeError::Validation(format!(
                "Invalid approval token decoded payload structure: {e}"
            ))
        })?
    };

    // 3. Verify asymmetric Ed25519 signature
    let canonical_bytes = token.payload.to_canonical_bytes()?;
    let sig_bytes_vec = hex::decode(&token.signature).ok_or_else(|| {
        LocalForgeError::Validation("Invalid approval signature encoding: must be valid hex".into())
    })?;

    if sig_bytes_vec.len() != 64 {
        return Err(LocalForgeError::Validation(format!(
            "Invalid approval signature length: expected 64 bytes (128 hex chars), found {}",
            sig_bytes_vec.len()
        )));
    }

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes_vec);
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key.verify(&canonical_bytes, &signature).map_err(|_| {
        LocalForgeError::Validation(
            "Authorization signature verification failed: token was not signed by the verified Product Owner public key".into(),
        )
    })?;

    // 4. Verify Expiration and Clock Skew
    let now = Utc::now().timestamp();
    if token.payload.expires_at < now {
        return Err(LocalForgeError::Validation(format!(
            "Authorization expired: token expired at {} (current time: {})",
            token.payload.expires_at, now
        )));
    }
    if token.payload.issued_at > now + 60 {
        return Err(LocalForgeError::Validation(format!(
            "Authorization rejected: token issued_at ({}) is in the future",
            token.payload.issued_at
        )));
    }

    // 5. Verify Scope
    if token.payload.scope != expected_scope {
        return Err(LocalForgeError::Validation(format!(
            "Authorization scope mismatch: token authorizes scope '{}', expected '{}'",
            token.payload.scope, expected_scope
        )));
    }

    // 6. Verify Exact Database Path
    let canonical_target =
        dunce::canonicalize(expected_db_path).unwrap_or_else(|_| expected_db_path.to_path_buf());
    let token_db_path = PathBuf::from(&token.payload.database_path);
    let canonical_token_path = dunce::canonicalize(&token_db_path).unwrap_or(token_db_path);

    if canonical_target != canonical_token_path {
        return Err(LocalForgeError::Validation(format!(
            "Authorization target mismatch: token authorizes database '{}', actual target is '{}'",
            canonical_token_path.display(),
            canonical_target.display()
        )));
    }

    // 7. Verify Activation Snapshot SHA-256 Checksum (Finding 04)
    if !token
        .payload
        .activation_snapshot_sha256
        .eq_ignore_ascii_case(expected_snapshot_sha256)
    {
        return Err(LocalForgeError::Validation(format!(
            "Authorization checksum mismatch: token binds to activation snapshot SHA-256 '{}', actual snapshot is '{}'",
            token.payload.activation_snapshot_sha256, expected_snapshot_sha256
        )));
    }

    // 8. Verify Exact Project Assertions (Finding 04: Never rely on project count alone)
    if token.payload.expected_projects.len() != expected_projects.len() {
        return Err(LocalForgeError::Validation(format!(
            "Authorization projects mismatch: token specifies {} projects, expected {}",
            token.payload.expected_projects.len(),
            expected_projects.len()
        )));
    }

    for expected in expected_projects {
        let matched = token
            .payload
            .expected_projects
            .iter()
            .find(|p| p.id == expected.id);

        match matched {
            Some(token_proj) => {
                if token_proj.name != expected.name {
                    return Err(LocalForgeError::Validation(format!(
                        "Project '{}' name mismatch: token specifies '{}', snapshot has '{}'",
                        expected.id, token_proj.name, expected.name
                    )));
                }
                if token_proj.path != expected.path {
                    return Err(LocalForgeError::Validation(format!(
                        "Project '{}' path mismatch: token specifies '{}', snapshot has '{}'",
                        expected.id, token_proj.path, expected.path
                    )));
                }
                if token_proj.volume_uuid != expected.volume_uuid {
                    return Err(LocalForgeError::Validation(format!(
                        "Project '{}' volume UUID mismatch: token specifies '{}', snapshot has '{}'",
                        expected.id, token_proj.volume_uuid, expected.volume_uuid
                    )));
                }
                if token_proj.document_root != expected.document_root {
                    return Err(LocalForgeError::Validation(format!(
                        "Project '{}' document root mismatch: token specifies '{}', snapshot has '{}'",
                        expected.id, token_proj.document_root, expected.document_root
                    )));
                }
                if token_proj.php_version != expected.php_version {
                    return Err(LocalForgeError::Validation(format!(
                        "Project '{}' PHP version mismatch: token specifies '{}', snapshot has '{}'",
                        expected.id, token_proj.php_version, expected.php_version
                    )));
                }
                if token_proj.assigned_loopback_port != expected.assigned_loopback_port {
                    return Err(LocalForgeError::Validation(format!(
                        "Project '{}' port mismatch: token authorizes port {}, expected {}",
                        expected.id,
                        token_proj.assigned_loopback_port,
                        expected.assigned_loopback_port
                    )));
                }
                if token_proj.name == "Projetest" && token_proj.php_version != "8.4" {
                    return Err(LocalForgeError::Validation(
                        "PO-U5-01 VIOLATION: Projetest must strictly remain pinned to PHP 8.4"
                            .into(),
                    ));
                }
            }
            None => {
                return Err(LocalForgeError::Validation(format!(
                    "Authorization project missing: token does not authorize project '{}'",
                    expected.id
                )));
            }
        }
    }

    // 9. Cross-Process Atomic Replay Protection (Finding 02)
    let parent = consumed_ledger_path.parent().ok_or_else(|| {
        LocalForgeError::DatabaseError("Invalid consumed approvals ledger path".into())
    })?;
    std::fs::create_dir_all(parent).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to create security dir: {e}"))
    })?;

    // Open ledger file with read/write and 0600 mode
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    #[cfg(unix)]
    use std::os::unix::io::AsRawFd;

    let mut ledger_opts = OpenOptions::new();
    ledger_opts.read(true).write(true).create(true);
    #[cfg(unix)]
    ledger_opts.mode(0o600);

    let mut ledger_file = ledger_opts.open(consumed_ledger_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "Failed to open consumed approvals ledger '{}': {e}",
            consumed_ledger_path.display()
        ))
    })?;

    // Ensure permissions 0600 even if file already existed
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ =
            std::fs::set_permissions(consumed_ledger_path, std::fs::Permissions::from_mode(0o600));
    }

    // Symlink and ownership verification
    let meta = std::fs::symlink_metadata(consumed_ledger_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to read ledger metadata: {e}"))
    })?;
    if meta.file_type().is_symlink() {
        return Err(LocalForgeError::DatabaseError(format!(
            "Security violation: replay ledger '{}' is a symbolic link",
            consumed_ledger_path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let current_uid = unsafe { libc::geteuid() };
        if meta.uid() != current_uid {
            return Err(LocalForgeError::DatabaseError(format!(
                "Security violation: replay ledger ownership mismatch: expected UID {current_uid}, got {}",
                meta.uid()
            )));
        }
    }

    // Acquire exclusive OS-level advisory lock (LOCK_EX)
    #[cfg(unix)]
    let _lock_guard = {
        let fd = ledger_file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX) };
        if ret != 0 {
            return Err(LocalForgeError::DatabaseError(
                "Failed to acquire exclusive lock on replay ledger".into(),
            ));
        }
        FlockGuard(fd)
    };

    // Read and validate all existing ledger records
    ledger_file.seek(std::io::SeekFrom::Start(0)).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to seek replay ledger to start: {e}"))
    })?;

    let reader = BufReader::new(&ledger_file);
    for (line_idx, line_res) in reader.lines().enumerate() {
        let line = line_res.map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to read replay ledger line: {e}"))
        })?;
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        // Fail closed on malformed or corrupted records
        let rec: ConsumedRecord = serde_json::from_str(trimmed_line).map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "Security violation: malformed or corrupted replay ledger record at line {}: {e}",
                line_idx + 1
            ))
        })?;

        if rec.token_id == token.payload.token_id {
            return Err(LocalForgeError::Validation(format!(
                "Security violation: approval token '{}' has already been consumed at {} (replay rejected)",
                token.payload.token_id, rec.consumed_at
            )));
        }
    }

    // 10. Record consumed token atomically and sync to disk before migration
    let new_record = ConsumedRecord {
        token_id: token.payload.token_id.clone(),
        consumed_at: Utc::now().to_rfc3339(),
        scope: token.payload.scope.clone(),
        activation_snapshot_sha256: token.payload.activation_snapshot_sha256.clone(),
    };
    let record_line = serde_json::to_string(&new_record).map_err(|e| {
        LocalForgeError::Serialization(format!("Failed to serialize consumed record: {e}"))
    })?;

    ledger_file.seek(std::io::SeekFrom::End(0)).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to seek replay ledger to end: {e}"))
    })?;

    writeln!(ledger_file, "{}", record_line).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to write to consumed approvals ledger: {e}"))
    })?;

    // Force durable write to disk before migration begins
    ledger_file.sync_all().map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "Failed to sync consumed approvals ledger to disk: {e}"
        ))
    })?;

    let prefix = if token.signature.len() >= 16 {
        token.signature[..16].to_string()
    } else {
        token.signature.clone()
    };

    Ok(ApprovalEvidence {
        token_id: token.payload.token_id,
        scope: token.payload.scope,
        activation_snapshot_sha256: token.payload.activation_snapshot_sha256,
        consumed_at: new_record.consumed_at,
        signature_prefix: prefix,
    })
}

fn base64_decode(input: &str) -> std::result::Result<Vec<u8>, ()> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;

    for &b in input.as_bytes() {
        if b == b'=' {
            break;
        }
        let val = TABLE.iter().position(|&x| x == b).ok_or(())? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn sample_assertions() -> Vec<ProjectSnapshotAssertion> {
        vec![
            ProjectSnapshotAssertion {
                id: "proj_e92e56cd0e0c4c36952804a35b3587a3".into(),
                name: "r1_acceptance_project".into(),
                path:
                    "/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project"
                        .into(),
                volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65".into(),
                document_root: "public".into(),
                php_version: "8.5".into(),
                local_domain: "r1-acceptance.localforge".into(),
                assigned_loopback_port: 18081,
            },
            ProjectSnapshotAssertion {
                id: "proj_db5472bb224c4fd0960ff69f793590a7".into(),
                name: "Projetest".into(),
                path: "/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/Projetest".into(),
                volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65".into(),
                document_root: "public".into(),
                php_version: "8.4".into(),
                local_domain: "projetest.localforge".into(),
                assigned_loopback_port: 18082,
            },
        ]
    }

    fn enroll_test_trust_anchor(dir: &Path, signing_key: &SigningKey) -> PathBuf {
        let anchor_path = dir.join("security").join("trust_anchor.json");
        let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
        crate::trust_anchor::enroll_trust_anchor(
            &anchor_path,
            &pubkey_hex,
            crate::trust_anchor::HUMAN_ENROLLMENT_PHRASE,
            false,
        )
        .unwrap();
        anchor_path
    }

    #[test]
    fn test_valid_asymmetric_approval_issuance_and_consumption() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"dummy db bytes").unwrap();
        let ledger_path = dir.path().join("security").join("consumed.jsonl");

        let signing_key = SigningKey::from_bytes(&[42u8; 32]);
        let anchor_path = enroll_test_trust_anchor(dir.path(), &signing_key);

        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256:
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            scope: "migrate-v2-loopback-ports".into(),
            expected_projects: sample_assertions(),
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        let token = MigrationApprovalToken::issue_with_signing_key(&signing_key, payload).unwrap();
        let token_json = token.to_json().unwrap();

        let evidence = verify_and_consume_approval(
            &token_json,
            &db_path,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            &anchor_path,
        )
        .unwrap();

        assert_eq!(evidence.scope, "migrate-v2-loopback-ports");
        assert!(ledger_path.exists());

        // Replay of the exact same token MUST fail closed
        let replay_err = verify_and_consume_approval(
            &token_json,
            &db_path,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();

        assert!(replay_err.to_string().contains("already been consumed"));
    }

    fn resolve_openssl_cli() -> Option<String> {
        let candidates = [
            "/opt/homebrew/bin/openssl",
            "/opt/homebrew/opt/openssl@3/bin/openssl",
            "/usr/local/opt/openssl@3/bin/openssl",
            "/usr/local/bin/openssl",
            "/usr/bin/openssl",
            "openssl",
        ];
        for candidate in candidates {
            if let Ok(out) = std::process::Command::new(candidate)
                .arg("version")
                .output()
            {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.contains("OpenSSL") {
                    return Some(candidate.to_string());
                }
            }
        }
        None
    }

    #[test]
    fn test_task1_disposable_ed25519_python_rust_interoperability() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"dummy db bytes").unwrap();
        let ledger_path = dir.path().join("security").join("consumed.jsonl");

        // 1. Generate a temporary Ed25519 key pair
        let key_path = dir.path().join("disposable_key.pem");
        let openssl_bin = resolve_openssl_cli();

        let (pub_hex, raw_seed): (String, Option<[u8; 32]>) = if let Some(ref bin) = openssl_bin {
            let status = std::process::Command::new(bin)
                .args(["genpkey", "-algorithm", "Ed25519", "-out"])
                .arg(&key_path)
                .status()
                .expect("Failed to execute openssl genpkey");
            assert!(status.success());

            let pubout = std::process::Command::new(bin)
                .args(["pkey", "-in"])
                .arg(&key_path)
                .args(["-pubout", "-outform", "DER"])
                .output()
                .expect("Failed to get pubkey via openssl");
            assert!(pubout.status.success());
            let der = pubout.stdout;
            let raw_pub = &der[der.len() - 32..];
            (hex::encode(raw_pub), None)
        } else {
            // Fallback: generate via ed25519-dalek from deterministic disposable seed
            let disposable_seed: [u8; 32] = [
                0x24, 0x5a, 0x71, 0x18, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
                0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
                0xdd, 0xee, 0xff, 0x01,
            ];
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&disposable_seed);
            let verifying_key = signing_key.verifying_key();
            let pub_hex = hex::encode(verifying_key.to_bytes());
            (pub_hex, Some(disposable_seed))
        };

        // Enroll trust anchor with disposable public key
        let anchor_path = dir.path().join("security").join("trust_anchor.json");
        std::fs::create_dir_all(anchor_path.parent().unwrap()).unwrap();
        crate::trust_anchor::enroll_trust_anchor(
            &anchor_path,
            &pub_hex,
            crate::trust_anchor::HUMAN_ENROLLMENT_PHRASE,
            false,
        )
        .unwrap();

        // 2. Construct test MigrationApprovalPayload matching Rust struct exactly
        let token_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let activation_sha =
            "2832b96f7a69fbb56d866cbe9bc259d379a6582a777bd45a333f161496f1e694".to_string();
        let scope = "migrate-v2-loopback-ports".to_string();
        let now = chrono::Utc::now().timestamp();
        let issued_at = now;
        let expires_at = now + 3600;

        let proj1 = crate::approval::ProjectSnapshotAssertion {
            id: "proj_db5472bb224c4fd0960ff69f793590a7".into(),
            name: "Projetest".into(),
            path: "/Volumes/Mac Harici Disk/Projetest".into(),
            volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65".into(),
            document_root: "public".into(),
            php_version: "8.4".into(),
            local_domain: "projetest.test".into(),
            assigned_loopback_port: 18082,
        };

        let proj2 = crate::approval::ProjectSnapshotAssertion {
            id: "proj_e92e56cd0e0c4c36952804a35b3587a3".into(),
            name: "r1_acceptance_project".into(),
            path: "/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project"
                .into(),
            volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65".into(),
            document_root: "public".into(),
            php_version: "8.5".into(),
            local_domain: "r1_acceptance_project.test".into(),
            assigned_loopback_port: 18081,
        };

        // Notice: pass projects intentionally in REVERSE order to test deterministic sorting
        let rust_payload = MigrationApprovalPayload {
            token_id: token_id.clone(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: activation_sha.clone(),
            scope: scope.clone(),
            expected_projects: vec![proj2.clone(), proj1.clone()],
            issued_at,
            expires_at,
        };

        // 3. Canonicalize the payload in Python & 4. Canonicalize in Rust
        let py_canonical_bin = dir.path().join("py_canonical.bin");
        let rust_canonical_bin = dir.path().join("rust_canonical.bin");
        let token_json_path = dir.path().join("token.json");
        let expired_token_json_path = dir.path().join("expired_token.json");

        // 4. Canonicalize in Rust using MigrationApprovalPayload::to_canonical_bytes()
        let rust_canonical_bytes = rust_payload.to_canonical_bytes().unwrap();
        std::fs::write(&rust_canonical_bin, &rust_canonical_bytes).unwrap();

        // 3. Canonicalize in Python (with project sorting and strict formatting)
        let openssl_arg = openssl_bin.unwrap_or_default();
        let seed_hex = raw_seed.map(hex::encode).unwrap_or_default();

        let py_script = format!(
            r#"
import json, subprocess, hashlib, os, tempfile

# RFC 8032 pure-Python fallback for Ed25519 signing
q = 2**255 - 19
l = 2**252 + 27742317777372353535851937790883648493
d = -121665 * pow(121666, q - 2, q)
I = pow(2, (q - 1) // 4, q)

def xrecover(y):
    xx = (y * y - 1) * pow(d * y * y + 1, q - 2, q)
    x = pow(xx, (q + 3) // 8, q)
    if (x * x - xx) % q != 0:
        x = (x * I) % q
    if x % 2 != 0:
        x = q - x
    return x

By = 4 * pow(5, q - 2, q)
Bx = xrecover(By)
B = [Bx % q, By % q]

def edwards(P, Q):
    x1, y1 = P[0], P[1]
    x2, y2 = Q[0], Q[1]
    x3 = (x1 * y2 + x2 * y1) * pow(1 + d * x1 * x2 * y1 * y2, q - 2, q)
    y3 = (y1 * y2 + x1 * x2) * pow(1 - d * x1 * x2 * y1 * y2, q - 2, q)
    return [x3 % q, y3 % q]

def scalarmult(P, e):
    if e == 0:
        return [0, 1]
    Q = scalarmult(P, e // 2)
    Q = edwards(Q, Q)
    if e & 1:
        Q = edwards(Q, P)
    return Q

def encodepoint(P):
    x, y = P[0], P[1]
    bits = [(y >> i) & 1 for i in range(255)] + [x & 1]
    return bytes(sum([bits[i * 8 + j] << j for j in range(8)]) for i in range(32))

def rfc8032_sign(m, sk_bytes):
    h = hashlib.sha512(sk_bytes).digest()
    a = 2**254 + sum(2**i * ((h[i // 8] >> (i % 8)) & 1) for i in range(3, 254))
    A = scalarmult(B, a)
    pk = encodepoint(A)
    r = int.from_bytes(hashlib.sha512(h[32:] + m).digest(), 'little')
    R = scalarmult(B, r)
    k = int.from_bytes(hashlib.sha512(encodepoint(R) + pk + m).digest(), 'little')
    S = (r + k * a) % l
    return (encodepoint(R) + S.to_bytes(32, 'little')).hex()

openssl_bin = "{}"
key_pem = "{}"
seed_hex = "{}"

def sign_data(msg_bytes):
    if openssl_bin and os.path.exists(key_pem):
        try:
            with tempfile.NamedTemporaryFile('wb', delete=False) as tf:
                tf.write(msg_bytes)
                tf_in = tf.name
            tf_out = tf_in + '.sig'
            subprocess.check_call([
                openssl_bin, 'pkeyutl', '-sign',
                '-inkey', key_pem,
                '-rawin',
                '-in', tf_in,
                '-out', tf_out
            ])
            with open(tf_out, 'rb') as f:
                s = f.read().hex()
            os.unlink(tf_in)
            os.unlink(tf_out)
            return s
        except Exception:
            pass
    if seed_hex:
        return rfc8032_sign(msg_bytes, bytes.fromhex(seed_hex))
    raise RuntimeError("Cannot sign")

# Input projects in REVERSE order to test canonical sorting
input_projects = [
    {{
        "id": "proj_e92e56cd0e0c4c36952804a35b3587a3",
        "name": "r1_acceptance_project",
        "path": "/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project",
        "volume_uuid": "7E26530E-5179-414C-81B5-6FBDB429FC65",
        "document_root": "public",
        "php_version": "8.5",
        "local_domain": "r1_acceptance_project.test",
        "assigned_loopback_port": 18081
    }},
    {{
        "id": "proj_db5472bb224c4fd0960ff69f793590a7",
        "name": "Projetest",
        "path": "/Volumes/Mac Harici Disk/Projetest",
        "volume_uuid": "7E26530E-5179-414C-81B5-6FBDB429FC65",
        "document_root": "public",
        "php_version": "8.4",
        "local_domain": "projetest.test",
        "assigned_loopback_port": 18082
    }}
]

# Canonicalize exactly matching Rust contract
sorted_projects = sorted(input_projects, key=lambda p: p["id"])
payload = {{
    "token_id": "{}",
    "database_path": "{}",
    "activation_snapshot_sha256": "{}".lower(),
    "scope": "{}",
    "expected_projects": sorted_projects,
    "issued_at": {},
    "expires_at": {}
}}

canonical_bytes = json.dumps(payload, separators=(',', ':')).encode('utf-8')
with open('{}', 'wb') as f:
    f.write(canonical_bytes)

token = {{
    "payload": payload,
    "signature": sign_data(canonical_bytes)
}}

with open('{}', 'w') as f:
    json.dump(token, f, indent=2)

# Also generate properly signed expired token
expired_payload = dict(payload)
expired_payload["issued_at"] = payload["issued_at"] - 7200
expired_payload["expires_at"] = payload["issued_at"] - 3600
expired_bytes = json.dumps(expired_payload, separators=(',', ':')).encode('utf-8')
expired_token = {{
    "payload": expired_payload,
    "signature": sign_data(expired_bytes)
}}

with open('{}', 'w') as f:
    json.dump(expired_token, f, indent=2)
"#,
            openssl_arg,
            key_path.display(),
            seed_hex,
            token_id,
            db_path.display(),
            activation_sha,
            scope,
            issued_at,
            expires_at,
            py_canonical_bin.display(),
            token_json_path.display(),
            expired_token_json_path.display(),
        );

        let py_output = std::process::Command::new("python3")
            .arg("-c")
            .arg(&py_script)
            .output()
            .expect("Failed to run python canonicalization and signing");

        if !py_output.status.success() {
            panic!(
                "Python script failed: {}",
                String::from_utf8_lossy(&py_output.stderr)
            );
        }

        // 5. Compare both byte sequences for exact equality
        let py_canonical_bytes = std::fs::read(&py_canonical_bin).unwrap();
        assert_eq!(
            py_canonical_bytes, rust_canonical_bytes,
            "Python and Rust canonical JSON byte sequences MUST be byte-for-byte identical"
        );

        // 6. Read signed token and verify signature using Rust ed25519_dalek directly
        let token_str = std::fs::read_to_string(&token_json_path).unwrap();
        let token: MigrationApprovalToken = serde_json::from_str(&token_str).unwrap();

        // 7. Verify signature using Rust ed25519_dalek
        let raw_pubkey = hex::decode(&pub_hex).expect("Valid pubkey hex");
        let pubkey_bytes: [u8; 32] = raw_pubkey.as_slice().try_into().unwrap();
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pubkey_bytes).unwrap();

        let sig_bytes = hex::decode(&token.signature).expect("Valid sig hex");
        let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().unwrap();
        let dalek_sig = ed25519_dalek::Signature::from_bytes(&sig_arr);

        verifying_key
            .verify_strict(&rust_canonical_bytes, &dalek_sig)
            .expect("Rust ed25519_dalek MUST verify the signature generated by Python/OpenSSL");

        let expected_assertions = vec![proj1.clone(), proj2.clone()];

        // Full end-to-end verification and consumption through verify_and_consume_approval
        let evidence = verify_and_consume_approval(
            &token_str,
            &db_path,
            &activation_sha,
            &scope,
            &expected_assertions,
            &ledger_path,
            &anchor_path,
        )
        .expect("Full verification pipeline MUST accept the valid token");
        assert_eq!(evidence.scope, "migrate-v2-loopback-ports");

        // 8. Test negative cases
        // 8a. Wrong signature
        let mut tampered_sig_token = token.clone();
        let mut bad_sig_bytes = sig_bytes.clone();
        bad_sig_bytes[0] ^= 0xff;
        tampered_sig_token.signature = hex::encode(bad_sig_bytes);
        let err = verify_and_consume_approval(
            &serde_json::to_string(&tampered_sig_token).unwrap(),
            &db_path,
            &activation_sha,
            &scope,
            &expected_assertions,
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("Authorization signature verification failed"));

        // 8b. Changed project port (18082 -> 18099)
        let mut altered_port_assertions = expected_assertions.clone();
        altered_port_assertions[0].assigned_loopback_port = 18099;
        let err = verify_and_consume_approval(
            &token_str,
            &db_path,
            &activation_sha,
            &scope,
            &altered_port_assertions,
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();
        assert!(err.to_string().contains("port mismatch"));

        // 8c. Changed SHA-256
        let err = verify_and_consume_approval(
            &token_str,
            &db_path,
            "0000000000000000000000000000000000000000000000000000000000000000",
            &scope,
            &expected_assertions,
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();
        assert!(err.to_string().contains("Authorization checksum mismatch"));

        // 8d. Expired token
        let expired_token_str = std::fs::read_to_string(&expired_token_json_path).unwrap();
        let err = verify_and_consume_approval(
            &expired_token_str,
            &db_path,
            &activation_sha,
            &scope,
            &expected_assertions,
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();
        assert!(err.to_string().contains("Authorization expired"));

        // 8e. Altered scope
        let err = verify_and_consume_approval(
            &token_str,
            &db_path,
            &activation_sha,
            "unauthorized-scope",
            &expected_assertions,
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();
        assert!(err.to_string().contains("Authorization scope mismatch"));

        // 9. Verify deterministic project ordering by ID
        // Shuffling the input order in Rust must yield the exact same bytes
        let shuffled_rust_payload = MigrationApprovalPayload {
            token_id: token_id.clone(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: activation_sha.clone(),
            scope: scope.clone(),
            expected_projects: vec![proj1.clone(), proj2.clone()], // proj1 first
            issued_at,
            expires_at,
        };
        assert_eq!(
            shuffled_rust_payload.to_canonical_bytes().unwrap(),
            rust_canonical_bytes,
            "Deterministic sorting must guarantee identical canonical bytes regardless of project declaration order"
        );
    }

    #[test]
    fn test_concurrent_token_consumption_race_condition_protection() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"dummy db bytes").unwrap();
        let ledger_path = Arc::new(dir.path().join("security").join("consumed.jsonl"));

        let signing_key = SigningKey::from_bytes(&[101u8; 32]);
        let anchor_path = Arc::new(enroll_test_trust_anchor(dir.path(), &signing_key));

        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            expected_projects: sample_assertions(),
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        let token = MigrationApprovalToken::issue_with_signing_key(&signing_key, payload).unwrap();
        let token_json = Arc::new(token.to_json().unwrap());
        let db_path_arc = Arc::new(db_path);

        // Spawn 8 concurrent threads attempting to consume the exact same token
        let mut handles = Vec::new();
        for _ in 0..8 {
            let t_json = Arc::clone(&token_json);
            let t_db = Arc::clone(&db_path_arc);
            let t_ledger = Arc::clone(&ledger_path);
            let t_anchor = Arc::clone(&anchor_path);

            handles.push(std::thread::spawn(move || {
                verify_and_consume_approval(
                    &t_json,
                    &t_db,
                    "abc",
                    "migrate-v2-loopback-ports",
                    &sample_assertions(),
                    &t_ledger,
                    &t_anchor,
                )
            }));
        }

        let mut successes = 0;
        let mut rejections = 0;

        for h in handles {
            match h.join().unwrap() {
                Ok(_) => successes += 1,
                Err(err) => {
                    assert!(err.to_string().contains("already been consumed"));
                    rejections += 1;
                }
            }
        }

        // Exactly ONE thread must succeed; all 7 others must be rejected
        assert_eq!(
            successes, 1,
            "Only 1 concurrent caller should consume the token"
        );
        assert_eq!(
            rejections, 7,
            "All 7 concurrent callers should be rejected by atomic lock"
        );
    }

    #[test]
    fn test_corrupted_replay_ledger_fails_closed() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("security").join("consumed.jsonl");
        std::fs::create_dir_all(ledger_path.parent().unwrap()).unwrap();

        // Write malformed corrupted JSON line
        std::fs::write(&ledger_path, b"{\"corrupt\": true\nINVALID_JSON_HERE\n").unwrap();

        let signing_key = SigningKey::from_bytes(&[77u8; 32]);
        let anchor_path = enroll_test_trust_anchor(dir.path(), &signing_key);

        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            expected_projects: sample_assertions(),
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        let token = MigrationApprovalToken::issue_with_signing_key(&signing_key, payload).unwrap();
        let token_json = token.to_json().unwrap();

        let err = verify_and_consume_approval(
            &token_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("malformed or corrupted replay ledger"));
    }

    #[test]
    fn test_agent_cannot_self_approve_without_po_private_key() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        // The true Product Owner keypair
        let po_signing_key = SigningKey::from_bytes(&[99u8; 32]);
        let anchor_path = enroll_test_trust_anchor(dir.path(), &po_signing_key);

        // An automated agent tries to generate its OWN keypair to self-approve
        let rogue_agent_key = SigningKey::from_bytes(&[66u8; 32]);

        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            expected_projects: sample_assertions(),
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        // Rogue agent signs with its own key
        let rogue_token =
            MigrationApprovalToken::issue_with_signing_key(&rogue_agent_key, payload).unwrap();
        let rogue_json = rogue_token.to_json().unwrap();

        // The executor verifier uses the true PO public key from trust anchor
        let err = verify_and_consume_approval(
            &rogue_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();

        assert!(err.to_string().contains("signature verification failed"));
    }

    #[test]
    fn test_projetest_php_mutation_fails_closed() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        let signing_key = SigningKey::from_bytes(&[55u8; 32]);
        let anchor_path = enroll_test_trust_anchor(dir.path(), &signing_key);

        let mut mutated_projects = sample_assertions();
        // Mutate Projetest PHP to 8.5 (PO-U5-01 violation)
        mutated_projects[1].php_version = "8.5".into();

        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            expected_projects: mutated_projects.clone(),
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        let token = MigrationApprovalToken::issue_with_signing_key(&signing_key, payload).unwrap();
        let token_json = token.to_json().unwrap();

        let err = verify_and_consume_approval(
            &token_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &mutated_projects,
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();

        assert!(err.to_string().contains("PO-U5-01 VIOLATION"));
    }

    #[test]
    fn test_attacker_key_substitution_fails_closed() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        // 1. Enrolled PO key
        let po_signing_key = SigningKey::from_bytes(&[12u8; 32]);
        let anchor_path = enroll_test_trust_anchor(dir.path(), &po_signing_key);

        // 2. Attacker generates their own keypair
        let attacker_signing_key = SigningKey::from_bytes(&[99u8; 32]);
        let attacker_pubkey_hex = hex::encode(attacker_signing_key.verifying_key().to_bytes());

        // 3. Attacker attempts to inject environment variable LOCALFORGE_PO_PUBLIC_KEY
        std::env::set_var("LOCALFORGE_PO_PUBLIC_KEY", &attacker_pubkey_hex);

        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            activation_snapshot_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            expected_projects: sample_assertions(),
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };
        let attacker_token =
            MigrationApprovalToken::issue_with_signing_key(&attacker_signing_key, payload).unwrap();
        let attacker_token_json = attacker_token.to_json().unwrap();

        // Verification MUST fail closed because load_verified_trust_anchor explicitly rejects LOCALFORGE_PO_PUBLIC_KEY
        let err = verify_and_consume_approval(
            &attacker_token_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            &anchor_path,
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("LOCALFORGE_PO_PUBLIC_KEY")
                || err.to_string().contains("signature verification failed"),
            "Expected env var rejection or signature verification failure, got: {err}"
        );

        // Clean up env var
        std::env::remove_var("LOCALFORGE_PO_PUBLIC_KEY");

        // 4. Attacker attempts to overwrite existing trust anchor without human replacement phrase
        let replace_err = crate::trust_anchor::enroll_trust_anchor(
            &anchor_path,
            &attacker_pubkey_hex,
            "ATTACKER UNAUTHORIZED OVERWRITE",
            true,
        )
        .unwrap_err();
        assert!(replace_err
            .to_string()
            .contains("Human authorization phrase"));

        // 5. Attacker attempts to replace without --replace flag
        let no_replace_err = crate::trust_anchor::enroll_trust_anchor(
            &anchor_path,
            &attacker_pubkey_hex,
            crate::trust_anchor::HUMAN_ENROLLMENT_PHRASE,
            false,
        )
        .unwrap_err();
        assert!(no_replace_err.to_string().contains("already enrolled"));

        // 6. Pointing to nonexistent trust anchor fails closed
        let missing_anchor_path = dir.path().join("nonexistent_anchor.json");
        let missing_err = verify_and_consume_approval(
            &attacker_token_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            &missing_anchor_path,
        )
        .unwrap_err();
        assert!(
            missing_err
                .to_string()
                .contains("Trust anchor unconfigured")
                || missing_err.to_string().contains("Trust anchor not found")
        );
    }
}
