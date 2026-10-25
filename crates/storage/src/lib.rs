pub mod approval;
pub mod barrier;
pub mod db;
pub mod paths;
pub mod rollback;
pub mod snapshot;

pub use approval::{
    compute_hmac_sha256, verify_and_consume_approval, ApprovalEvidence, MigrationApprovalPayload,
    MigrationApprovalToken, ProjectPortApproval,
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
