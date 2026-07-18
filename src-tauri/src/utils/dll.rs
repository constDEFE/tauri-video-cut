use std::iter::once;
use std::os::windows::ffi::OsStrExt;

pub fn add_lib_to_dll_search_path() {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let lib_dir = exe_dir.join("lib");
            if lib_dir.exists() {
                let wide: Vec<u16> = lib_dir.as_os_str().encode_wide().chain(once(0)).collect();
                unsafe extern "system" {
                    fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
                }
                unsafe {
                    SetDllDirectoryW(wide.as_ptr());
                }
            }
        }
    }
}
