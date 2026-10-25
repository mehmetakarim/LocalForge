use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
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
    po_pubkey_override: Option<&[u8]>,
) -> Result<ApprovalEvidence> {
    // 1. Resolve Product Owner public key (from override, environment, or config file)
    let pubkey_bytes: [u8; 32] = if let Some(pk_slice) = po_pubkey_override {
        if pk_slice.len() != 32 {
            return Err(LocalForgeError::Validation(
                "Invalid Product Owner public key override length: expected 32 bytes".into(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(pk_slice);
        arr
    } else if let Ok(env_pk_hex) = std::env::var("LOCALFORGE_PO_PUBLIC_KEY") {
        let trimmed = env_pk_hex.trim();
        let bytes = hex::decode(trimmed).ok_or_else(|| {
            LocalForgeError::Validation("LOCALFORGE_PO_PUBLIC_KEY must be valid 64-char hex".into())
        })?;
        if bytes.len() != 32 {
            return Err(LocalForgeError::Validation(
                "LOCALFORGE_PO_PUBLIC_KEY must be exactly 32 bytes (64 hex characters)".into(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    } else {
        return Err(LocalForgeError::Validation(
            "Authorization rejected: Product Owner public key not configured (LOCALFORGE_PO_PUBLIC_KEY required)".into(),
        ));
    };

    let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes).map_err(|e| {
        LocalForgeError::Validation(format!("Invalid Product Owner Ed25519 public key: {e}"))
    })?;

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

    #[test]
    fn test_valid_asymmetric_approval_issuance_and_consumption() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"dummy db bytes").unwrap();
        let ledger_path = dir.path().join("security").join("consumed.jsonl");

        let signing_key = SigningKey::from_bytes(&[42u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes = verifying_key.to_bytes();

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
            Some(&pubkey_bytes),
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
            Some(&pubkey_bytes),
        )
        .unwrap_err();

        assert!(replay_err.to_string().contains("already been consumed"));
    }

    #[test]
    fn test_concurrent_token_consumption_race_condition_protection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"dummy db bytes").unwrap();
        let ledger_path = Arc::new(dir.path().join("security").join("consumed.jsonl"));

        let signing_key = SigningKey::from_bytes(&[101u8; 32]);
        let pubkey_bytes = Arc::new(signing_key.verifying_key().to_bytes());

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
            let t_pk = Arc::clone(&pubkey_bytes);

            handles.push(std::thread::spawn(move || {
                verify_and_consume_approval(
                    &t_json,
                    &t_db,
                    "abc",
                    "migrate-v2-loopback-ports",
                    &sample_assertions(),
                    &t_ledger,
                    Some(&*t_pk),
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
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("security").join("consumed.jsonl");
        std::fs::create_dir_all(ledger_path.parent().unwrap()).unwrap();

        // Write malformed corrupted JSON line
        std::fs::write(&ledger_path, b"{\"corrupt\": true\nINVALID_JSON_HERE\n").unwrap();

        let signing_key = SigningKey::from_bytes(&[77u8; 32]);
        let pubkey_bytes = signing_key.verifying_key().to_bytes();

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
            Some(&pubkey_bytes),
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("malformed or corrupted replay ledger"));
    }

    #[test]
    fn test_agent_cannot_self_approve_without_po_private_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        // The true Product Owner keypair
        let po_signing_key = SigningKey::from_bytes(&[99u8; 32]);
        let po_pubkey_bytes = po_signing_key.verifying_key().to_bytes();

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

        // The executor verifier uses the true PO public key
        let err = verify_and_consume_approval(
            &rogue_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &sample_assertions(),
            &ledger_path,
            Some(&po_pubkey_bytes),
        )
        .unwrap_err();

        assert!(err.to_string().contains("signature verification failed"));
    }

    #[test]
    fn test_projetest_php_mutation_fails_closed() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        let signing_key = SigningKey::from_bytes(&[55u8; 32]);
        let pubkey_bytes = signing_key.verifying_key().to_bytes();

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
            Some(&pubkey_bytes),
        )
        .unwrap_err();

        assert!(err.to_string().contains("PO-U5-01 VIOLATION"));
    }
}
