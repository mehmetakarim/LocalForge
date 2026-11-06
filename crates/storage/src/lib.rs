pub mod approval;
pub mod barrier;
pub mod db;
pub mod paths;
pub mod rollback;
pub mod snapshot;
pub mod trust_anchor;

pub use approval::{
    verify_and_consume_approval, ApprovalEvidence, MigrationApprovalPayload,
    MigrationApprovalToken, ProjectSnapshotAssertion,
};
pub use barrier::VolumeBarrier;
pub use db::{
    validate_loopback_port_index, IndexValidationStatus, MetadataStore, MigrationReport,
    ProjectRecord,
};
pub use paths::sanitize_project_path;
pub use rollback::{
    execute_gated_migration_lifecycle, execute_isolated_quarantine_and_restore,
    verify_writer_quiescence, FailureInjectionPoint, GatedMigrationResult, RollbackConfig,
};
pub use snapshot::{
    compute_file_sha256, create_wal_safe_snapshot, inspect_wal_database_preflight,
    ProjectSnapshotRecord, WalSnapshotAudit,
};
pub use trust_anchor::{
    enroll_trust_anchor, load_verified_trust_anchor, PoTrustAnchor, HUMAN_ENROLLMENT_PHRASE,
    HUMAN_REPLACEMENT_PHRASE,
};

#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
