use std::path::Path;

/// Evaluates whether a relative file path should be strictly excluded from remote deployment.
pub fn is_excluded_from_deployment(rel_path: &Path) -> bool {
    let path_str = rel_path.to_string_lossy().to_lowercase();
    let file_name = rel_path
        .file_name()
        .map(|f| f.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // 1. Source control
    if path_str.starts_with(".git/") || path_str == ".git" || file_name == ".gitignore" {
        return true;
    }

    // 2. Environment files (allow .env.example)
    if (file_name == ".env" || file_name.starts_with(".env.")) && file_name != ".env.example" {
        return true;
    }

    // 3. Private keys, certificates, SSH tokens
    if file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".p12")
        || file_name.starts_with("id_rsa")
        || file_name.starts_with("id_ed25519")
        || file_name.starts_with("id_ecdsa")
        || file_name == "known_hosts"
    {
        return true;
    }

    // 4. OS and editor metadata
    if file_name == ".ds_store" || file_name == "thumbs.db" || path_str.starts_with(".idea/") || path_str.starts_with(".vscode/") {
        return true;
    }

    // 5. LocalForge metadata, runtime data, ledgers, previews & sockets
    if path_str.starts_with(".localforge/")
        || path_str == ".localforge"
        || path_str.ends_with(".sock")
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclusions() {
        assert!(is_excluded_from_deployment(Path::new(".env")));
        assert!(is_excluded_from_deployment(Path::new(".env.production")));
        assert!(!is_excluded_from_deployment(Path::new(".env.example"))); // Example is permitted

        assert!(is_excluded_from_deployment(Path::new(".git/config")));
        assert!(is_excluded_from_deployment(Path::new("server.key")));
        assert!(is_excluded_from_deployment(Path::new("certs/cert.pem")));
        assert!(is_excluded_from_deployment(Path::new(".DS_Store")));

        assert!(!is_excluded_from_deployment(Path::new("index.php")));
        assert!(!is_excluded_from_deployment(Path::new("public/style.css")));
    }
}
