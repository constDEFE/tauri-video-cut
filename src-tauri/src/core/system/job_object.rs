use crate::error::AppError;
use crate::logger::{log_error, log_warn};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::Arc;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};

#[derive(Clone)]
pub struct ProcessManager {
    job: Arc<OwnedHandle>,
}

impl ProcessManager {
    pub fn new() -> Option<Self> {
        unsafe {
            let raw = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());

            if raw.is_null() {
                log_error!(last_error = GetLastError(), "CreateJobObjectW failed");

                return None;
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = Default::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            if SetInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) == 0
            {
                log_error!(
                    last_error = GetLastError(),
                    "SetInformationJobObject failed"
                );

                let _ = CloseHandle(raw as _);
                return None;
            }

            Some(Self {
                job: Arc::new(OwnedHandle::from_raw_handle(raw as _)),
            })
        }
    }

    pub fn attach(&self, child: &tokio::process::Child) -> Result<(), AppError> {
        let raw_child = child
            .raw_handle()
            .ok_or_else(|| AppError::FFmpegError("child.raw_handle() returned null".into()))?;

        unsafe {
            if AssignProcessToJobObject(self.job.as_raw_handle() as HANDLE, raw_child as HANDLE)
                == 0
            {
                let err = GetLastError();

                log_error!(
                    pid = child.id().unwrap_or(0),
                    last_error = err,
                    "AssignProcessToJobObject failed"
                );

                Err(AppError::FFmpegError(format!(
                    "AssignProcessToJobObject failed ({err})"
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn kill_all(&self) {
        unsafe {
            let result = TerminateJobObject(self.job.as_raw_handle() as HANDLE, 1);
            if result == 0 {
                let err = GetLastError();

                log_error!(last_error = err, "TerminateJobObject failed");
            } else {
                log_warn!("All job object processes terminated via kill_all");
            }
        }
    }
}
