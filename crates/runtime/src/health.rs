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

/// Probes loopback HTTP server with specific Host header and path.
/// Returns (status_code, response_body).
pub fn probe_http_health(
    port: u16,
    host_header: &str,
    path: &str,
) -> crate::error::Result<(u16, String)> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("127.0.0.1:{port}");
    let socket_addr = addr
        .parse()
        .map_err(|e| crate::error::RuntimeError::ServiceUnhealthy {
            service: "nginx".into(),
            reason: format!("Invalid address {addr}: {e}"),
        })?;

    let mut stream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2)).map_err(|e| {
            crate::error::RuntimeError::ServiceUnhealthy {
                service: "nginx".into(),
                reason: format!("Failed to connect to gateway at {addr}: {e}"),
            }
        })?;

    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host_header}\r\nUser-Agent: LocalForge-Probe/1.0\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes)?;
    let response_text = String::from_utf8_lossy(&response_bytes).to_string();

    if let Some(first_line) = response_text.lines().next() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(code) = parts[1].parse::<u16>() {
                return Ok((code, response_text));
            }
        }
    }

    Err(crate::error::RuntimeError::ServiceUnhealthy {
        service: "nginx".into(),
        reason: format!("Malformed HTTP response: {response_text}"),
    })
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
