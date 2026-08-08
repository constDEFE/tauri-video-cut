use std::fs::File;
use std::io::BufWriter;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::FromRawHandle;
use std::path::Path;
use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_TEMPORARY, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    LockFileEx, OPEN_ALWAYS, OPEN_EXISTING, UnlockFileEx,
};
use windows_sys::Win32::System::IO::OVERLAPPED;

#[derive(Clone, Copy)]
struct SendHandle(HANDLE);
unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

fn to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub struct CacheLock {
    handle: SendHandle,
}

impl CacheLock {
    pub fn try_acquire(lock_path: &Path) -> Option<Self> {
        let wide = to_wide(lock_path);
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null_mut(),
                OPEN_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut ov: OVERLAPPED = unsafe { std::mem::zeroed() };

        if unsafe {
            LockFileEx(
                handle,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                u32::MAX,
                0,
                &mut ov,
            )
        } != 0
        {
            Some(Self {
                handle: SendHandle(handle),
            })
        } else {
            unsafe { CloseHandle(handle) };
            None
        }
    }
}

impl Drop for CacheLock {
    fn drop(&mut self) {
        let mut ov: OVERLAPPED = unsafe { std::mem::zeroed() };
        unsafe {
            UnlockFileEx(self.handle.0, 0, u32::MAX, 0, &mut ov);
            CloseHandle(self.handle.0);
        }
    }
}

pub fn create_staging(part_path: &Path) -> Option<BufWriter<File>> {
    let wide = to_wide(part_path);
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ,
            std::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_TEMPORARY,
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return None;
    }

    Some(BufWriter::new(unsafe { File::from_raw_handle(handle) }))
}

pub fn open_shared_read(path: &Path) -> Option<File> {
    let wide = to_wide(path);
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return None;
    }

    Some(unsafe { File::from_raw_handle(handle) })
}
