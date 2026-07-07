use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crate::error::AppError;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};

/// Thread-safe wrapper for the Windows Job Object.
/// Wraps the raw HANDLE in an Arc so it's shared across threads and
/// closed exactly once when the last reference drops.
///
/// Uses an AtomicUsize flag to track whether the handle has been closed,
/// preventing use-after-close bugs since CloseHandle doesn't invalidate the value.
pub struct ProcessManager {
    job: Arc<HANDLE>,
    /// 1 = valid, 0 = closed. Checked atomically before every operation.
    is_valid: Arc<AtomicUsize>,
}

// SAFETY: INVALID_HANDLE_VALUE is -1 (0xFFFFFFFFFFFFFFFF) which is also
// a valid pointer value on some systems. We cast to usize for comparison
// since we know the handle came from CreateJobObjectW.
//
// NOTE: After CloseHandle, the handle value is not automatically invalidated.
// We must track validity ourselves via an AtomicUsize flag.
fn is_valid_handle(h: HANDLE) -> bool {
    h as usize != 0 && h as usize != 0xFFFFFFFFFFFFFFFF
}

// SAFETY: HANDLE is just a pointer-sized value. Sharing it across threads
// is safe because the underlying kernel object handles its own lifetime,
// and we close it exactly once via Drop on Arc<HANDLE>.
unsafe impl Send for ProcessManager {}
unsafe impl Sync for ProcessManager {}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        // Share both the job handle AND the validity flag across clones.
        // When any clone is dropped, all clones see the handle as invalid.
        ProcessManager {
            job: Arc::clone(&self.job),
            is_valid: Arc::clone(&self.is_valid),
        }
    }
}

impl ProcessManager {
    /// Creates a new job object. Returns None if creation fails.
    /// The KILL_ON_JOB_CLOSE flag ensures child processes are terminated
    /// when this process exits or crashes.
    pub fn new() -> Option<Self> {
        unsafe {
            let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            // CreateJobObjectW returns INVALID_HANDLE_VALUE on failure.
            if !is_valid_handle(job) {
                return None;
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();

            // Magic flag: Tells Windows Kernel to kill all assigned processes if app exits/crashes
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            // SetInformationJobObject returns FALSE on failure - silently ignore,
            // child processes will still be tracked but won't auto-kill on crash.
            let _ = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            Some(ProcessManager {
                job: Arc::new(job),
                is_valid: Arc::new(AtomicUsize::new(1)),
            })
        }
    }

    /// Attach any spawned process handle to the OS job.
    /// Returns an error if the job handle is invalid or the assignment fails.
    pub fn attach(&self, child: &tokio::process::Child) -> Result<(), AppError> {
        // Check validity flag first - fast path for closed handles.
        if self.is_valid.load(Ordering::SeqCst) == 0 {
            return Err(AppError::FFmpegError(
                "job handle is invalid (closed)".to_string(),
            ));
        }

        let raw_handle = child.raw_handle().ok_or_else(|| {
            AppError::FFmpegError("child.raw_handle() returned null".to_string())
        })?;

        unsafe {
            // AssignProcessToJobObject returns FALSE on failure.
            if AssignProcessToJobObject(*self.job, raw_handle as HANDLE) == 0 {
                let err = windows_sys::Win32::Foundation::GetLastError();
                Err(AppError::FFmpegError(format!(
                    "AssignProcessToJobObject failed, GetLastError={}",
                    err
                )))
            } else {
                Ok(())
            }
        }
    }

    /// Cancel/Kill all running ffmpeg/ffprobe tasks immediately.
    /// No-op if job handle is invalid (already closed).
    pub fn kill_all(&self) {
        // Check validity flag first - fast path for closed handles.
        if self.is_valid.load(Ordering::SeqCst) == 0 {
            return;
        }
        unsafe {
            TerminateJobObject(*self.job, 1);
        }
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        // Mark as closed BEFORE closing the handle to prevent use-after-close.
        // The AtomicUsize flag is checked atomically by other threads.
        self.is_valid.store(0, Ordering::SeqCst);

        // Close the job handle exactly once when the last Arc reference drops.
        // This ensures child processes are cleaned up by the kernel if we exit.
        if is_valid_handle(*self.job) {
            unsafe { CloseHandle(*self.job); }
        }
    }
}
