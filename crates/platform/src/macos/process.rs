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
                // Check start time: exactly matches
                if identity.start_time_secs != expected_start_secs
                    || identity.start_time_usecs != expected_start_usecs
                {
                    return false;
                }

                // Check executable path
                let canonical_expected = dunce::canonicalize(expected_executable)
                    .unwrap_or_else(|_| expected_executable.to_path_buf());

                // Match exact path or file name if canonical paths resolve symlinks
                identity.executable_path() == canonical_expected
                    || identity.executable_path().file_name() == canonical_expected.file_name()
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
                // Process already terminated
                return Ok(());
            }
        };

        // Strict PID reuse check
        if identity.start_time_secs != expected_start_secs
            || identity.start_time_usecs != expected_start_usecs
        {
            return Err(LocalForgeError::PlatformError(format!(
                "PID reuse mismatch detected for PID {pid}: expected start {expected_start_secs}.{expected_start_usecs}, found {}.{}",
                identity.start_time_secs, identity.start_time_usecs
            )));
        }

        // Strict executable check
        let canonical_expected = dunce::canonicalize(expected_executable)
            .unwrap_or_else(|_| expected_executable.to_path_buf());
        if identity.executable_path() != canonical_expected
            && identity.executable_path().file_name() != canonical_expected.file_name()
        {
            return Err(LocalForgeError::PlatformError(format!(
                "Executable mismatch detected for PID {pid}: expected '{}', found '{}'",
                canonical_expected.display(),
                identity.executable_path().display()
            )));
        }

        let pgid = identity.pgid;

        // Stage 1: Send SIGTERM to process group or single process
        unsafe {
            if pgid > 1 && pgid == pid {
                libc::killpg(pgid as i32, libc::SIGTERM);
            } else {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }

        // Wait up to grace_period_secs
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(grace_period_secs);
        while start.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(100));
            match self.inspect_process(pid)? {
                Some(current) => {
                    // Check if PID was recycled while waiting
                    if current.start_time_secs != expected_start_secs
                        || current.start_time_usecs != expected_start_usecs
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

        // Stage 2: Bounded escalation to SIGKILL if still alive and identity still matches
        if let Some(current) = self.inspect_process(pid)? {
            if current.start_time_secs == expected_start_secs
                && current.start_time_usecs == expected_start_usecs
            {
                unsafe {
                    if pgid > 1 && pgid == pid {
                        libc::killpg(pgid as i32, libc::SIGKILL);
                    } else {
                        libc::kill(pid as i32, libc::SIGKILL);
                    }
                }
            }
        }

        Ok(())
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
}
