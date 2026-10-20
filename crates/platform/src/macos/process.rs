use crate::traits::{ProcessIdentity, ProcessManager};
use localforge_core::{LocalForgeError, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const PROC_PIDT_BSDINFO: i32 = 3;
pub const PROC_PIDPATHINFO_MAXSIZE: usize = 4096;

extern "C" {
    fn proc_pidpath(pid: i32, buffer: *mut u8, buffersize: u32) -> i32;
    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut libc::c_void,
        buffersize: i32,
    ) -> i32;
}

pub struct MacOSProcessManager;

impl MacOSProcessManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOSProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager for MacOSProcessManager {
    fn inspect_process(&self, pid: u32) -> Result<Option<ProcessIdentity>> {
        if pid == 0 {
            return Ok(None);
        }

        let mut bsdinfo = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
        let expected_size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
        let ret = unsafe {
            proc_pidinfo(
                pid as i32,
                PROC_PIDT_BSDINFO,
                0,
                bsdinfo.as_mut_ptr() as *mut libc::c_void,
                expected_size,
            )
        };

        if ret < expected_size {
            // Process does not exist (ESRCH) or access denied
            return Ok(None);
        }

        let bsdinfo = unsafe { bsdinfo.assume_init() };

        // Process UID verification: Must belong to current user
        let current_uid = unsafe { libc::getuid() };
        if bsdinfo.pbi_uid != current_uid {
            // Process belongs to another user (e.g. root or system user); fail closed
            return Ok(None);
        }

        let mut path_buf = [0u8; PROC_PIDPATHINFO_MAXSIZE];
        let path_ret = unsafe {
            proc_pidpath(
                pid as i32,
                path_buf.as_mut_ptr(),
                PROC_PIDPATHINFO_MAXSIZE as u32,
            )
        };

        let executable_path = if path_ret > 0 {
            let path_bytes = &path_buf[..path_ret as usize];
            let path_str = String::from_utf8_lossy(path_bytes);
            PathBuf::from(path_str.into_owned())
        } else {
            // If path could not be resolved, fallback to comm name
            let comm_bytes: Vec<u8> = bsdinfo
                .pbi_comm
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8)
                .collect();
            let comm_str = String::from_utf8_lossy(&comm_bytes);
            PathBuf::from(comm_str.into_owned())
        };

        // Canonicalize path if possible
        let canonical_path = dunce::canonicalize(&executable_path).unwrap_or(executable_path);

        Ok(Some(ProcessIdentity {
            pid,
            pgid: bsdinfo.pbi_pgid,
            uid: bsdinfo.pbi_uid,
            std_path: canonical_path,
            start_time_secs: bsdinfo.pbi_start_tvsec,
            start_time_usecs: bsdinfo.pbi_start_tvusec as u32,
        }))
    }

    fn verify_process_identity(
        &self,
        pid: u32,
        expected_executable: &Path,
        expected_start_secs: u64,
        expected_start_usecs: u32,
    ) -> bool {
        match self.inspect_process(pid) {
            Ok(Some(identity)) => {
                // 1. Verify owner UID matches current process UID
                let current_uid = unsafe { libc::getuid() };
                if identity.uid != current_uid {
                    return false;
                }

                // 2. Strict start time verification: exactly matches
                if identity.start_time_secs != expected_start_secs
                    || identity.start_time_usecs != expected_start_usecs
                {
                    return false;
                }

                // 3. Strict canonical executable path verification
                let canonical_expected = dunce::canonicalize(expected_executable)
                    .unwrap_or_else(|_| expected_executable.to_path_buf());

                // Strictly match canonical path (no unsafe basename fallback!)
                identity.executable_path() == canonical_expected
            }
            _ => false,
        }
    }

    fn terminate_process_group_verified(
        &self,
        pid: u32,
        expected_executable: &Path,
        expected_start_secs: u64,
        expected_start_usecs: u32,
        grace_period_secs: u64,
    ) -> Result<()> {
        let identity = match self.inspect_process(pid)? {
            Some(id) => id,
            None => {
                // Process already terminated or does not belong to current user
                return Ok(());
            }
        };

        // 1. Verify owner UID
        let current_uid = unsafe { libc::getuid() };
        if identity.uid != current_uid {
            return Err(LocalForgeError::PlatformError(format!(
                "UID mismatch for PID {pid}: process owner UID {} does not match current user {current_uid}",
                identity.uid
            )));
        }

        // 2. Strict PID reuse check
        if identity.start_time_secs != expected_start_secs
            || identity.start_time_usecs != expected_start_usecs
        {
            return Err(LocalForgeError::PlatformError(format!(
                "PID reuse mismatch detected for PID {pid}: expected start {expected_start_secs}.{expected_start_usecs}, found {}.{}",
                identity.start_time_secs, identity.start_time_usecs
            )));
        }

        // 3. Strict canonical executable path check (NO unsafe basename fallback)
        let canonical_expected = dunce::canonicalize(expected_executable)
            .unwrap_or_else(|_| expected_executable.to_path_buf());
        if identity.executable_path() != canonical_expected {
            return Err(LocalForgeError::PlatformError(format!(
                "Executable mismatch detected for PID {pid}: expected '{}', found '{}'",
                canonical_expected.display(),
                identity.executable_path().display()
            )));
        }

        // RESIDUAL TOCTOU RISK & PLATFORM LIMITATIONS:
        // On Unix platforms (including macOS), there is a theoretical microsecond TOCTOU window
        // between inspecting a PID via proc_pidinfo and issuing the kill/killpg syscall.
        // Pure atomic signal-by-token requires kernel pidfds (Linux) or Mach task ports,
        // which macOS does not permit unprivileged processes to acquire for child daemons.
        // LocalForge mitigates this window by:
        // 1. Verifying UID matches current user.
        // 2. Verifying exact kernel start timestamp (seconds + microseconds).
        // 3. Verifying the canonical binary path (strictly rejecting basename-only matches).
        // 4. Verifying process group containment (preventing accidental self-group signaling).
        // 5. Checking signal syscall return values and errno.

        let pgid = identity.pgid;
        let my_pgid = unsafe { libc::getpgrp() as u32 };

        // Process group targeting:
        // Only target process group if pgid > 1, pgid == pid (process is group leader),
        // and pgid is NOT our own process group (to prevent killing LocalForge itself!).
        let use_killpg = pgid > 1 && pgid == pid && pgid != my_pgid;

        // Stage 1: Send SIGTERM and check syscall result
        let res = unsafe {
            if use_killpg {
                libc::killpg(pgid as i32, libc::SIGTERM)
            } else {
                libc::kill(pid as i32, libc::SIGTERM)
            }
        };

        if res != 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ESRCH) {
                return Err(LocalForgeError::PlatformError(format!(
                    "Failed to send SIGTERM to PID {pid} (pgid: {pgid}): {err}"
                )));
            }
        }

        // Wait up to grace_period_secs with polling
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(grace_period_secs);
        while start.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(100));
            match self.inspect_process(pid)? {
                Some(current) => {
                    // Check if PID was recycled while waiting
                    if current.start_time_secs != expected_start_secs
                        || current.start_time_usecs != expected_start_usecs
                        || current.uid != current_uid
                    {
                        // Process exited and PID was recycled by another process! Do not kill!
                        return Ok(());
                    }
                }
                None => {
                    // Process exited cleanly
                    return Ok(());
                }
            }
        }

        // Stage 2: Bounded escalation to SIGKILL if still alive and identity still strictly matches
        if let Some(current) = self.inspect_process(pid)? {
            if current.start_time_secs == expected_start_secs
                && current.start_time_usecs == expected_start_usecs
                && current.uid == current_uid
                && current.executable_path() == canonical_expected
            {
                let res = unsafe {
                    if use_killpg {
                        libc::killpg(pgid as i32, libc::SIGKILL)
                    } else {
                        libc::kill(pid as i32, libc::SIGKILL)
                    }
                };
                if res != 0 {
                    let err = std::io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::ESRCH) {
                        return Err(LocalForgeError::PlatformError(format!(
                            "Failed to send SIGKILL to PID {pid}: {err}"
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn find_listening_pids(&self, port: u16) -> Result<Vec<u32>> {
        let output = std::process::Command::new("/usr/sbin/lsof")
            .arg("-nP")
            .arg(format!("-iTCP:{port}"))
            .arg("-sTCP:LISTEN")
            .arg("-t")
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => {
                match std::process::Command::new("lsof")
                    .arg("-nP")
                    .arg(format!("-iTCP:{port}"))
                    .arg("-sTCP:LISTEN")
                    .arg("-t")
                    .output()
                {
                    Ok(o) => o,
                    Err(e) => {
                        return Err(LocalForgeError::PlatformError(format!(
                            "Failed to execute lsof: {e}"
                        )))
                    }
                }
            }
        };

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut pids = Vec::new();
        for line in text.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                if !pids.contains(&pid) {
                    pids.push(pid);
                }
            }
        }
        Ok(pids)
    }

    fn get_process_cmdline(&self, pid: u32) -> Result<Option<Vec<String>>> {
        if pid == 0 {
            return Ok(None);
        }
        const CTL_KERN: libc::c_int = 1;
        const KERN_PROCARGS2: libc::c_int = 49;
        let mut mib = [CTL_KERN, KERN_PROCARGS2, pid as libc::c_int];
        let mut size: libc::size_t = 0;

        unsafe {
            if libc::sysctl(
                mib.as_mut_ptr(),
                3,
                std::ptr::null_mut(),
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
                || size == 0
            {
                return Ok(None);
            }

            let mut buffer = vec![0u8; size];
            if libc::sysctl(
                mib.as_mut_ptr(),
                3,
                buffer.as_mut_ptr() as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                return Ok(None);
            }

            if buffer.len() < std::mem::size_of::<libc::c_int>() {
                return Ok(None);
            }

            let argc = *(buffer.as_ptr() as *const libc::c_int) as usize;
            let mut offset = std::mem::size_of::<libc::c_int>();

            // Skip executable path (null-terminated string)
            while offset < buffer.len() && buffer[offset] != 0 {
                offset += 1;
            }
            // Skip trailing null bytes after exec path
            while offset < buffer.len() && buffer[offset] == 0 {
                offset += 1;
            }

            let mut args = Vec::with_capacity(argc);
            for _ in 0..argc {
                if offset >= buffer.len() {
                    break;
                }
                let start = offset;
                while offset < buffer.len() && buffer[offset] != 0 {
                    offset += 1;
                }
                let arg_bytes = &buffer[start..offset];
                args.push(String::from_utf8_lossy(arg_bytes).to_string());
                if offset < buffer.len() {
                    offset += 1; // skip null byte
                }
            }

            Ok(Some(args))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_current_process() {
        let pm = MacOSProcessManager::new();
        let pid = std::process::id();
        let res = pm.inspect_process(pid).unwrap();
        assert!(res.is_some());
        let id = res.unwrap();
        assert_eq!(id.pid, pid);
        assert!(id.start_time_secs > 0);
        assert!(!id.executable_path().as_os_str().is_empty());
    }

    #[test]
    fn test_pid_reuse_protection_rejects_mismatch() {
        let pm = MacOSProcessManager::new();
        let pid = std::process::id();
        let id = pm.inspect_process(pid).unwrap().unwrap();

        // Artificially alter start time
        let altered_start_secs = id.start_time_secs + 9999;
        let is_verified = pm.verify_process_identity(
            pid,
            id.executable_path(),
            altered_start_secs,
            id.start_time_usecs,
        );
        assert!(!is_verified, "PID reuse should be detected and rejected");

        // Attempting terminate with altered start time must fail closed
        let err = pm.terminate_process_group_verified(
            pid,
            id.executable_path(),
            altered_start_secs,
            id.start_time_usecs,
            1,
        );
        assert!(err.is_err(), "Must fail closed on start time mismatch");
    }

    #[test]
    fn test_executable_path_mismatch_fails_closed() {
        let pm = MacOSProcessManager::new();
        let pid = std::process::id();
        let id = pm.inspect_process(pid).unwrap().unwrap();

        let dummy_path = Path::new("/bin/ls");
        let is_verified =
            pm.verify_process_identity(pid, dummy_path, id.start_time_secs, id.start_time_usecs);
        assert!(!is_verified, "Different executable path must be rejected");

        let err = pm.terminate_process_group_verified(
            pid,
            dummy_path,
            id.start_time_secs,
            id.start_time_usecs,
            1,
        );
        assert!(err.is_err(), "Must fail closed on executable path mismatch");
    }

    #[test]
    fn test_basename_only_match_is_strictly_rejected() {
        let pm = MacOSProcessManager::new();
        let pid = std::process::id();
        let id = pm.inspect_process(pid).unwrap().unwrap();

        // Create a spoofed path with identical basename but different directory
        let fake_path = Path::new("/tmp").join(id.executable_path().file_name().unwrap());

        let is_verified =
            pm.verify_process_identity(pid, &fake_path, id.start_time_secs, id.start_time_usecs);
        assert!(
            !is_verified,
            "Basename-only matching must be strictly rejected (unsafe fallback removed)"
        );

        let err = pm.terminate_process_group_verified(
            pid,
            &fake_path,
            id.start_time_secs,
            id.start_time_usecs,
            1,
        );
        assert!(err.is_err(), "Must fail closed on non-canonical path match");
    }

    #[test]
    fn test_process_group_safety_avoids_self_group() {
        let pm = MacOSProcessManager::new();
        let pid = std::process::id();
        let id = pm.inspect_process(pid).unwrap().unwrap();
        let my_pgid = unsafe { libc::getpgrp() as u32 };

        // Process group of current process should match getpgrp()
        assert_eq!(id.pgid, my_pgid);
        // The manager's safety guard ensures that even if pgid == pid,
        // it will never call killpg on current test process group.
    }
}
