use localforge_core::{LocalForgeError, Result};
use std::path::{Path, PathBuf};

/// Sanitizes and canonicalizes a path, ensuring it strictly resides within `allowed_root`.
pub fn sanitize_project_path(project_root: &Path, relative_or_absolute: &Path) -> Result<PathBuf> {
    let canonical_root = dunce::canonicalize(project_root).map_err(LocalForgeError::Io)?;

    let target = if relative_or_absolute.is_absolute() {
        relative_or_absolute.to_path_buf()
    } else {
        canonical_root.join(relative_or_absolute)
    };

    // Normalize path components lexically to resolve '.' and '..'
    let mut normalized = PathBuf::new();
    for component in target.components() {
        match component {
            std::path::Component::Prefix(p) => normalized.push(p.as_os_str()),
            std::path::Component::RootDir => normalized.push("/"),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(c) => normalized.push(c),
        }
    }

    if !normalized.starts_with(&canonical_root) {
        return Err(LocalForgeError::PathTraversalViolation {
            path: normalized,
            root: canonical_root,
        });
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_path_sanitization_valid() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("public");
        std::fs::create_dir(&sub).unwrap();

        let res = sanitize_project_path(dir.path(), Path::new("public"));
        assert!(res.is_ok());
    }

    #[test]
    fn test_path_traversal_detection() {
        let dir = tempdir().unwrap();
        let outside = Path::new("../../etc/passwd");

        let res = sanitize_project_path(dir.path(), outside);
        assert!(res.is_err());
        match res.unwrap_err() {
            LocalForgeError::PathTraversalViolation { .. } => {}
            other => panic!("Expected PathTraversalViolation, got: {:?}", other),
        }
    }
}
