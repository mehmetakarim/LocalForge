use ed25519_dalek::VerifyingKey;
use localforge_core::{LocalForgeError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub const HUMAN_ENROLLMENT_PHRASE: &str = "I AUTHORIZE ENROLLMENT OF PRODUCT OWNER TRUST ANCHOR";
pub const HUMAN_REPLACEMENT_PHRASE: &str =
    "I AUTHORIZE REPLACEMENT OF EXISTING PRODUCT OWNER TRUST ANCHOR";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoTrustAnchor {
    pub key_id: String,
    pub algorithm: String,
    pub public_key_hex: String,
    pub fingerprint_sha256: String,
    pub enrolled_at: String,
    pub enrolled_by: String,
    pub human_authorization_phrase: String,
}

impl PoTrustAnchor {
    pub fn compute_fingerprint(pubkey_bytes: &[u8; 32]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pubkey_bytes);
        crate::approval::hex::encode(hasher.finalize())
    }
}

/// Enrolls a new Product Owner trust anchor or replaces an existing one with explicit confirmation.
///
/// Operational constraints & threat boundary:
/// 1. Enrollment requires the confirmation phrase: `I AUTHORIZE ENROLLMENT OF PRODUCT OWNER TRUST ANCHOR`.
/// 2. Key replacement requires the explicit phrase: `I AUTHORIZE REPLACEMENT OF EXISTING PRODUCT OWNER TRUST ANCHOR` and `replace = true`.
///    Note: These static confirmation phrases guard against accidental CLI invocation; they do NOT
///    constitute cryptographic human authentication against adversarial actors sharing the same OS user process context.
/// 3. Cannot be written to a symlink.
/// 4. File permissions are strictly enforced as 0600 on unix.
/// 5. Validates that the public key is exactly 32 bytes (64 hex characters) and forms a valid Ed25519 point.
pub fn enroll_trust_anchor(
    anchor_path: &Path,
    public_key_hex: &str,
    human_auth_phrase: &str,
    replace: bool,
) -> Result<PoTrustAnchor> {
    let trimmed_key = public_key_hex.trim();
    let pubkey_bytes_vec = crate::approval::hex::decode(trimmed_key).ok_or_else(|| {
        LocalForgeError::Validation(
            "Product Owner public key must be valid 64-character hex".into(),
        )
    })?;

    if pubkey_bytes_vec.len() != 32 {
        return Err(LocalForgeError::Validation(format!(
            "Product Owner public key must be exactly 32 bytes (64 hex chars), found {} bytes",
            pubkey_bytes_vec.len()
        )));
    }

    let mut pubkey_bytes = [0u8; 32];
    pubkey_bytes.copy_from_slice(&pubkey_bytes_vec);

    // Validate that it forms a valid Ed25519 verifying key
    VerifyingKey::from_bytes(&pubkey_bytes).map_err(|e| {
        LocalForgeError::Validation(format!("Invalid Ed25519 public key curve point: {e}"))
    })?;

    if anchor_path.exists() {
        let meta = std::fs::symlink_metadata(anchor_path).map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to read trust anchor metadata: {e}"))
        })?;
        if meta.file_type().is_symlink() {
            return Err(LocalForgeError::Validation(
                "Security violation: trust anchor target cannot be a symbolic link".into(),
            ));
        }

        if !replace {
            return Err(LocalForgeError::Validation(
                "Trust anchor already enrolled. Key replacement requires explicit human authorization phrase: 'I AUTHORIZE REPLACEMENT OF EXISTING PRODUCT OWNER TRUST ANCHOR' and replace flag set to true.".into(),
            ));
        }

        if human_auth_phrase.trim() != HUMAN_REPLACEMENT_PHRASE {
            return Err(LocalForgeError::Validation(format!(
                "Human authorization phrase mismatch for key replacement: expected '{HUMAN_REPLACEMENT_PHRASE}'"
            )));
        }
    } else if human_auth_phrase.trim() != HUMAN_ENROLLMENT_PHRASE {
        return Err(LocalForgeError::Validation(format!(
            "Human authorization phrase mismatch for key enrollment: expected '{HUMAN_ENROLLMENT_PHRASE}'"
        )));
    }

    let fingerprint = PoTrustAnchor::compute_fingerprint(&pubkey_bytes);
    let key_id = format!("po-ed25519-{}", &fingerprint[..12]);

    let anchor = PoTrustAnchor {
        key_id,
        algorithm: "ed25519".into(),
        public_key_hex: trimmed_key.to_lowercase(),
        fingerprint_sha256: fingerprint,
        enrolled_at: chrono::Utc::now().to_rfc3339(),
        enrolled_by: "Product Owner (Human Operator)".into(),
        human_authorization_phrase: human_auth_phrase.trim().into(),
    };

    let anchor_json = serde_json::to_string_pretty(&anchor).map_err(|e| {
        LocalForgeError::Serialization(format!("Failed to serialize trust anchor: {e}"))
    })?;

    if let Some(parent) = anchor_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "Failed to create trust anchor parent directory: {e}"
            ))
        })?;
    }

    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);

    let mut file = opts.open(anchor_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "Failed to open trust anchor file '{}' for writing: {e}",
            anchor_path.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(anchor_path, std::fs::Permissions::from_mode(0o600));
    }

    file.write_all(anchor_json.as_bytes()).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to write trust anchor data: {e}"))
    })?;
    file.sync_all().map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to sync trust anchor to disk: {e}"))
    })?;

    Ok(anchor)
}

/// Loads and strictly validates the enrolled Product Owner trust anchor.
///
/// Security constraints:
/// 1. The environment variable `LOCALFORGE_PO_PUBLIC_KEY` is explicitly DISALLOWED to prevent invoking actors from selecting the key.
/// 2. If the trust anchor does not exist or is invalid, fails closed.
/// 3. Rejects symbolic links.
/// 4. Validates ownership (UID) and enforces 0600 file permissions on unix.
/// 5. Validates cryptographic fingerprint match against public key bytes.
pub fn load_verified_trust_anchor(anchor_path: &Path) -> Result<(VerifyingKey, PoTrustAnchor)> {
    // 1. Strict defense: Disallow environment-provided public key override
    if std::env::var("LOCALFORGE_PO_PUBLIC_KEY").is_ok() {
        return Err(LocalForgeError::Validation(
            "Security violation: LOCALFORGE_PO_PUBLIC_KEY environment variable is strictly disallowed. The executor only trusts the enrolled Product Owner trust anchor.".into(),
        ));
    }

    // 2. Fail-closed check: Trust anchor file must exist
    if !anchor_path.exists() {
        return Err(LocalForgeError::Validation(format!(
            "Trust anchor unconfigured: Product Owner public key has not been enrolled at '{}'. Live activation is strictly prohibited without a verified trust anchor.",
            anchor_path.display()
        )));
    }

    // 3. Symlink and ownership verification
    let meta = std::fs::symlink_metadata(anchor_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to read trust anchor metadata: {e}"))
    })?;
    if meta.file_type().is_symlink() {
        return Err(LocalForgeError::Validation(format!(
            "Security violation: trust anchor '{}' is a symbolic link",
            anchor_path.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let current_uid = unsafe { libc::geteuid() };
        if meta.uid() != current_uid {
            return Err(LocalForgeError::Validation(format!(
                "Security violation: trust anchor ownership mismatch: expected UID {current_uid}, got {}",
                meta.uid()
            )));
        }

        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        if mode & 0o077 != 0 {
            return Err(LocalForgeError::Validation(format!(
                "Security violation: trust anchor permissions too open ({mode:o}), must be 0600",
            )));
        }
    }

    // 4. Parse trust anchor JSON
    let content = std::fs::read_to_string(anchor_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to read trust anchor file: {e}"))
    })?;

    let anchor: PoTrustAnchor = serde_json::from_str(&content).map_err(|e| {
        LocalForgeError::Validation(format!("Corrupt or malformed trust anchor JSON: {e}"))
    })?;

    if anchor.algorithm != "ed25519" {
        return Err(LocalForgeError::Validation(format!(
            "Unsupported trust anchor algorithm: '{}', expected 'ed25519'",
            anchor.algorithm
        )));
    }

    let pubkey_bytes_vec =
        crate::approval::hex::decode(&anchor.public_key_hex).ok_or_else(|| {
            LocalForgeError::Validation("Trust anchor public key hex is malformed".into())
        })?;

    if pubkey_bytes_vec.len() != 32 {
        return Err(LocalForgeError::Validation(format!(
            "Trust anchor public key length invalid: expected 32 bytes, found {}",
            pubkey_bytes_vec.len()
        )));
    }

    let mut pubkey_bytes = [0u8; 32];
    pubkey_bytes.copy_from_slice(&pubkey_bytes_vec);

    let expected_fingerprint = PoTrustAnchor::compute_fingerprint(&pubkey_bytes);
    if !anchor
        .fingerprint_sha256
        .eq_ignore_ascii_case(&expected_fingerprint)
    {
        return Err(LocalForgeError::Validation(format!(
            "Trust anchor fingerprint mismatch: recorded '{}', computed '{}'",
            anchor.fingerprint_sha256, expected_fingerprint
        )));
    }

    let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes).map_err(|e| {
        LocalForgeError::Validation(format!(
            "Invalid Ed25519 verifying key in trust anchor: {e}"
        ))
    })?;

    Ok((verifying_key, anchor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    #[test]
    fn test_trust_anchor_enrollment_and_verification_lifecycle() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let anchor_path = dir.path().join("trust_anchor.json");

        let signing_key = SigningKey::from_bytes(&[123u8; 32]);
        let pubkey_hex = crate::approval::hex::encode(signing_key.verifying_key().to_bytes());

        // 1. Enrollment requires exact human authorization phrase
        let err = enroll_trust_anchor(
            &anchor_path,
            &pubkey_hex,
            "random agent authorization string",
            false,
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("Human authorization phrase mismatch"));

        // 2. Successful enrollment
        let anchor =
            enroll_trust_anchor(&anchor_path, &pubkey_hex, HUMAN_ENROLLMENT_PHRASE, false).unwrap();

        assert_eq!(anchor.algorithm, "ed25519");
        assert_eq!(anchor.public_key_hex, pubkey_hex.to_lowercase());

        // 3. Load verified trust anchor
        let (vkey, loaded_anchor) = load_verified_trust_anchor(&anchor_path).unwrap();
        assert_eq!(vkey.to_bytes(), signing_key.verifying_key().to_bytes());
        assert_eq!(loaded_anchor.key_id, anchor.key_id);

        // 4. Replacement without replace flag fails closed
        let signing_key_2 = SigningKey::from_bytes(&[200u8; 32]);
        let pubkey_hex_2 = crate::approval::hex::encode(signing_key_2.verifying_key().to_bytes());

        let rep_err_1 =
            enroll_trust_anchor(&anchor_path, &pubkey_hex_2, HUMAN_ENROLLMENT_PHRASE, false)
                .unwrap_err();
        assert!(rep_err_1
            .to_string()
            .contains("Trust anchor already enrolled"));

        // 5. Replacement with wrong phrase fails closed
        let rep_err_2 =
            enroll_trust_anchor(&anchor_path, &pubkey_hex_2, HUMAN_ENROLLMENT_PHRASE, true)
                .unwrap_err();
        assert!(rep_err_2
            .to_string()
            .contains("Human authorization phrase mismatch for key replacement"));

        // 6. Legitimate replacement with exact replacement phrase succeeds
        let replaced_anchor =
            enroll_trust_anchor(&anchor_path, &pubkey_hex_2, HUMAN_REPLACEMENT_PHRASE, true)
                .unwrap();

        let (vkey_2, _) = load_verified_trust_anchor(&anchor_path).unwrap();
        assert_eq!(vkey_2.to_bytes(), signing_key_2.verifying_key().to_bytes());
        assert_ne!(replaced_anchor.key_id, anchor.key_id);
    }

    #[test]
    fn test_environment_variable_override_is_strictly_rejected() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let anchor_path = dir.path().join("trust_anchor.json");

        let signing_key = SigningKey::from_bytes(&[77u8; 32]);
        let pubkey_hex = crate::approval::hex::encode(signing_key.verifying_key().to_bytes());

        enroll_trust_anchor(&anchor_path, &pubkey_hex, HUMAN_ENROLLMENT_PHRASE, false).unwrap();

        // Attacker sets LOCALFORGE_PO_PUBLIC_KEY in environment
        std::env::set_var(
            "LOCALFORGE_PO_PUBLIC_KEY",
            "deadbeef00000000000000000000000000000000000000000000000000000000",
        );

        let err = load_verified_trust_anchor(&anchor_path).unwrap_err();
        std::env::remove_var("LOCALFORGE_PO_PUBLIC_KEY");

        assert!(err
            .to_string()
            .contains("LOCALFORGE_PO_PUBLIC_KEY environment variable is strictly disallowed"));
    }

    #[test]
    fn test_unconfigured_trust_anchor_fails_closed() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let missing_path = dir.path().join("missing_anchor.json");

        let err = load_verified_trust_anchor(&missing_path).unwrap_err();
        assert!(err.to_string().contains("Trust anchor unconfigured"));
    }

    #[test]
    fn test_tampered_trust_anchor_fingerprint_fails_closed() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let anchor_path = dir.path().join("trust_anchor.json");

        let signing_key = SigningKey::from_bytes(&[88u8; 32]);
        let pubkey_hex = crate::approval::hex::encode(signing_key.verifying_key().to_bytes());

        let mut anchor =
            enroll_trust_anchor(&anchor_path, &pubkey_hex, HUMAN_ENROLLMENT_PHRASE, false).unwrap();
        // Tamper with public key while keeping old fingerprint
        anchor.public_key_hex = "00".repeat(32);
        let tampered_json = serde_json::to_string_pretty(&anchor).unwrap();
        std::fs::write(&anchor_path, tampered_json).unwrap();

        let err = load_verified_trust_anchor(&anchor_path).unwrap_err();
        assert!(err.to_string().contains("fingerprint mismatch"));
    }
}
