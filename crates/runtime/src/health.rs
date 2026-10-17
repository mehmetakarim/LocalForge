use std::net::{SocketAddr, TcpListener};
use std::path::Path;

/// Checks if a TCP port on localhost (127.0.0.1) is free to bind.
pub fn is_port_available(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpListener::bind(addr).is_ok()
}

/// Finds the next available loopback port starting from `start`.
pub fn find_available_port(start: u16, max_attempts: u16) -> Option<u16> {
    (start..(start + max_attempts)).find(|&port| is_port_available(port))
}

/// Checks if a Unix domain socket exists and is responsive.
pub fn is_unix_socket_alive(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    // Attempt non-blocking connect via std::os::unix::net::UnixStream
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;
        UnixStream::connect(path).is_ok()
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_search() {
        let port = find_available_port(18080, 50);
        assert!(port.is_some());
        let found = port.unwrap();
        assert!(found >= 18080);
        assert!(is_port_available(found));
    }
}
