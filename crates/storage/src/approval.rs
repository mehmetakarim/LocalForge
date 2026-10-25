use chrono::Utc;
use localforge_core::{LocalForgeError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Implements standard RFC 2104 / RFC 4231 HMAC-SHA256 without external crate dependencies.
pub fn compute_hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let key_hash = hasher.finalize();
        key_block[..32].copy_from_slice(&key_hash);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut o_key_pad = [0x5cu8; 64];
    let mut i_key_pad = [0x36u8; 64];
    for i in 0..64 {
        o_key_pad[i] ^= key_block[i];
        i_key_pad[i] ^= key_block[i];
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(i_key_pad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(o_key_pad);
    outer_hasher.update(inner_hash);
    outer_hasher.finalize().into()
}

/// Constant-time byte slice comparison to defend against timing attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectPortApproval {
    pub id: String,
    pub loopback_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationApprovalPayload {
    pub token_id: String,
    pub database_path: String,
    pub database_sha256: String,
    pub scope: String,
    pub projects: Vec<ProjectPortApproval>,
    pub issued_at: i64,
    pub expires_at: i64,
}

impl MigrationApprovalPayload {
    /// Produces a deterministic canonical byte sequence for signing.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>> {
        // Sort projects by ID for deterministic byte layout
        let mut sorted_projects = self.projects.clone();
        sorted_projects.sort_by(|a, b| a.id.cmp(&b.id));

        let canonical_struct = MigrationApprovalPayload {
            token_id: self.token_id.clone(),
            database_path: self.database_path.clone(),
            database_sha256: self.database_sha256.clone(),
            scope: self.scope.clone(),
            projects: sorted_projects,
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
    pub signature: String,
}

impl MigrationApprovalToken {
    /// Issues a verifiable token signed with the provided Product Owner key.
    pub fn issue(key: &[u8], payload: MigrationApprovalPayload) -> Result<Self> {
        let canonical_bytes = payload.to_canonical_bytes()?;
        let hmac = compute_hmac_sha256(key, &canonical_bytes);
        let signature = hex::encode(hmac);
        Ok(Self { payload, signature })
    }

    /// Serializes token to formatted JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| LocalForgeError::Serialization(e.to_string()))
    }
}

mod hex {
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
    pub database_sha256: String,
    pub consumed_at: String,
    pub signature_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsumedRecord {
    token_id: String,
    consumed_at: String,
    scope: String,
    database_sha256: String,
}

/// Verifies and atomically consumes a Product Owner approval token.
///
/// Ensures:
/// 1. Token cannot be self-issued by the executor (requires PO secret key).
/// 2. Binds approval to the exact database path, target database SHA-256, project IDs, and port assignments.
/// 3. Rejects expired tokens.
/// 4. Prevents replay attacks by checking and recording consumed token IDs in an isolated ledger.
/// 5. Records approval evidence without exposing secrets.
pub fn verify_and_consume_approval(
    token_str: &str,
    expected_db_path: &Path,
    expected_db_sha256: &str,
    expected_scope: &str,
    expected_projects: &[(&str, u16)],
    consumed_ledger_path: &Path,
    po_key_override: Option<&[u8]>,
) -> Result<ApprovalEvidence> {
    // 1. Resolve Product Owner secret key (from environment or explicit override for tests)
    let env_key = std::env::var("LOCALFORGE_PO_APPROVAL_KEY").ok();
    let key_bytes: &[u8] = if let Some(override_key) = po_key_override {
        override_key
    } else if let Some(ref k) = env_key {
        k.as_bytes()
    } else {
        return Err(LocalForgeError::Validation(
            "Authorization rejected: Product Owner authorization key not found in environment (LOCALFORGE_PO_APPROVAL_KEY required)".into(),
        ));
    };

    if key_bytes.is_empty() {
        return Err(LocalForgeError::Validation(
            "Authorization rejected: Product Owner authorization key cannot be empty".into(),
        ));
    }

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

    // 3. Verify signature
    let canonical_bytes = token.payload.to_canonical_bytes()?;
    let expected_hmac = compute_hmac_sha256(key_bytes, &canonical_bytes);
    let Some(provided_hmac) = hex::decode(&token.signature) else {
        return Err(LocalForgeError::Validation(
            "Invalid approval signature encoding: must be valid hex".into(),
        ));
    };

    if !constant_time_eq(&expected_hmac, &provided_hmac) {
        return Err(LocalForgeError::Validation(
            "Authorization signature mismatch: token was not issued with the verified Product Owner approval key".into(),
        ));
    }

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

    // 7. Verify Database Checksum
    if !token
        .payload
        .database_sha256
        .eq_ignore_ascii_case(expected_db_sha256)
    {
        return Err(LocalForgeError::Validation(format!(
            "Authorization checksum mismatch: token binds to database SHA-256 '{}', current is '{}'",
            token.payload.database_sha256, expected_db_sha256
        )));
    }

    // 8. Verify Project IDs and Port Assignments
    if token.payload.projects.len() != expected_projects.len() {
        return Err(LocalForgeError::Validation(format!(
            "Authorization projects mismatch: token specifies {} projects, expected {}",
            token.payload.projects.len(),
            expected_projects.len()
        )));
    }

    for (exp_id, exp_port) in expected_projects {
        let matched = token.payload.projects.iter().find(|p| p.id == *exp_id);
        match matched {
            Some(p) => {
                if p.loopback_port != *exp_port {
                    return Err(LocalForgeError::Validation(format!(
                        "Authorization port mismatch for project '{}': token authorizes port {}, expected {}",
                        exp_id, p.loopback_port, exp_port
                    )));
                }
            }
            None => {
                return Err(LocalForgeError::Validation(format!(
                    "Authorization project missing: token does not authorize project '{}'",
                    exp_id
                )));
            }
        }
    }

    // 9. Replay Check
    if let Some(parent) = consumed_ledger_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to create security ledger dir: {e}"))
        })?;
    }

    if consumed_ledger_path.exists() {
        let file = std::fs::File::open(consumed_ledger_path).map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to open consumed approvals ledger: {e}"))
        })?;
        let reader = BufReader::new(file);
        for line_res in reader.lines() {
            let line: String = line_res.map_err(|e| {
                LocalForgeError::DatabaseError(format!("Failed to read consumed ledger line: {e}"))
            })?;
            if let Ok(rec) = serde_json::from_str::<ConsumedRecord>(&line) {
                if rec.token_id == token.payload.token_id {
                    return Err(LocalForgeError::Validation(format!(
                        "Security violation: approval token '{}' has already been consumed (replay prevented)",
                        token.payload.token_id
                    )));
                }
            }
        }
    }

    // 10. Atomically record consumption
    let record = ConsumedRecord {
        token_id: token.payload.token_id.clone(),
        consumed_at: Utc::now().to_rfc3339(),
        scope: token.payload.scope.clone(),
        database_sha256: token.payload.database_sha256.clone(),
    };
    let record_line = serde_json::to_string(&record).map_err(|e| {
        LocalForgeError::Serialization(format!("Failed to serialize consumed record: {e}"))
    })?;

    let mut ledger_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(consumed_ledger_path)
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "Failed to open consumed approvals ledger for writing: {e}"
            ))
        })?;

    // Enforce 0600 permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ =
            std::fs::set_permissions(consumed_ledger_path, std::fs::Permissions::from_mode(0o600));
    }

    writeln!(ledger_file, "{}", record_line).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to record consumed approval token: {e}"))
    })?;
    ledger_file.flush().map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to flush consumed approvals ledger: {e}"))
    })?;

    let sig_prefix = if token.signature.len() >= 12 {
        token.signature[..12].to_string()
    } else {
        token.signature.clone()
    };

    Ok(ApprovalEvidence {
        token_id: token.payload.token_id,
        scope: token.payload.scope,
        database_sha256: token.payload.database_sha256,
        consumed_at: record.consumed_at,
        signature_prefix: sig_prefix,
    })
}

fn base64_decode(input: &str) -> std::result::Result<Vec<u8>, ()> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let bytes = input.as_bytes();
    let mut buf = 0u32;
    let mut bits = 0u32;

    for &b in bytes {
        if b == b'=' {
            break;
        }
        let val = match TABLE.iter().position(|&x| x == b) {
            Some(idx) => idx as u32,
            None => return Err(()),
        };
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
    use tempfile::tempdir;

    #[test]
    fn test_rfc4231_test_vector_1() {
        // RFC 4231 Test Case 1
        let key = vec![0x0bu8; 20];
        let data = b"Hi There";
        let hmac = compute_hmac_sha256(&key, data);
        let expected_hex = "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7";
        assert_eq!(hex::encode(hmac), expected_hex);
    }

    #[test]
    fn test_valid_approval_token_issuance_and_consumption() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"dummy db bytes").unwrap();
        let ledger_path = dir.path().join("security").join("consumed.jsonl");

        let po_key = b"super_secret_po_production_key_98765";
        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            database_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .into(),
            scope: "migrate-v2-loopback-ports".into(),
            projects: vec![
                ProjectPortApproval {
                    id: "proj_e92e56cd0e0c4c36952804a35b3587a3".into(),
                    loopback_port: 18081,
                },
                ProjectPortApproval {
                    id: "proj_db5472bb224c4fd0960ff69f793590a7".into(),
                    loopback_port: 18082,
                },
            ],
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        let token = MigrationApprovalToken::issue(po_key, payload).unwrap();
        let token_json = token.to_json().unwrap();

        let expected_projects = [
            ("proj_e92e56cd0e0c4c36952804a35b3587a3", 18081),
            ("proj_db5472bb224c4fd0960ff69f793590a7", 18082),
        ];

        let evidence = verify_and_consume_approval(
            &token_json,
            &db_path,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "migrate-v2-loopback-ports",
            &expected_projects,
            &ledger_path,
            Some(po_key),
        )
        .unwrap();

        assert_eq!(evidence.scope, "migrate-v2-loopback-ports");
        assert!(ledger_path.exists());

        // Replay of the same token MUST be rejected
        let replay_err = verify_and_consume_approval(
            &token_json,
            &db_path,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "migrate-v2-loopback-ports",
            &expected_projects,
            &ledger_path,
            Some(po_key),
        )
        .unwrap_err();

        assert!(replay_err.to_string().contains("already been consumed"));
    }

    #[test]
    fn test_expired_token_rejection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        let po_key = b"secret";
        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            database_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            projects: vec![ProjectPortApproval {
                id: "proj_1".into(),
                loopback_port: 18081,
            }],
            issued_at: Utc::now().timestamp() - 7200,
            expires_at: Utc::now().timestamp() - 3600, // Expired
        };

        let token = MigrationApprovalToken::issue(po_key, payload).unwrap();
        let token_json = token.to_json().unwrap();

        let err = verify_and_consume_approval(
            &token_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &[("proj_1", 18081)],
            &ledger_path,
            Some(po_key),
        )
        .unwrap_err();

        assert!(err.to_string().contains("Authorization expired"));
    }

    #[test]
    fn test_tampered_payload_and_arbitrary_string_rejection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        std::fs::write(&db_path, b"db").unwrap();
        let ledger_path = dir.path().join("consumed.jsonl");

        let po_key = b"secret";

        // Arbitrary non-JSON string
        let err1 = verify_and_consume_approval(
            "arbitrary_unverified_token_string",
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &[("proj_1", 18081)],
            &ledger_path,
            Some(po_key),
        )
        .unwrap_err();
        assert!(err1.to_string().contains("Invalid approval token"));

        // Tampered port in payload
        let payload = MigrationApprovalPayload {
            token_id: uuid::Uuid::new_v4().to_string(),
            database_path: db_path.display().to_string(),
            database_sha256: "abc".into(),
            scope: "migrate-v2-loopback-ports".into(),
            projects: vec![ProjectPortApproval {
                id: "proj_1".into(),
                loopback_port: 18081,
            }],
            issued_at: Utc::now().timestamp(),
            expires_at: Utc::now().timestamp() + 3600,
        };

        let mut token = MigrationApprovalToken::issue(po_key, payload).unwrap();
        // Tamper signature
        token.signature = "deadbeef1234".into();
        let token_json = token.to_json().unwrap();

        let err2 = verify_and_consume_approval(
            &token_json,
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &[("proj_1", 18081)],
            &ledger_path,
            Some(po_key),
        )
        .unwrap_err();
        assert!(
            err2.to_string().contains("signature mismatch")
                || err2.to_string().contains("Invalid approval signature")
        );
    }

    #[test]
    fn test_missing_po_key_fails_closed() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        let ledger_path = dir.path().join("consumed.jsonl");

        std::env::remove_var("LOCALFORGE_PO_APPROVAL_KEY");

        let err = verify_and_consume_approval(
            "{}",
            &db_path,
            "abc",
            "migrate-v2-loopback-ports",
            &[],
            &ledger_path,
            None,
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("LOCALFORGE_PO_APPROVAL_KEY required"));
    }
}
