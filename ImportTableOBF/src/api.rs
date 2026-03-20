use std::ffi::CString;
use std::ptr::null_mut;
use std::os::raw::c_void;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use crate::hash_api;

pub unsafe fn resolve_api(hash: u32) -> *mut c_void {
    let module_name = CString::new("kernel32.dll").unwrap();
    let hmodule: HMODULE = GetModuleHandleA(&module_name);

    if hmodule.0 == 0 {
        return null_mut();
    }

    let funcs = [
        ("CreateToolhelp32Snapshot", hash_api("CreateToolhelp32Snapshot")),
        ("Process32FirstW", hash_api("Process32FirstW")),
        ("Process32NextW", hash_api("Process32NextW")),
    ];

    for (name, h) in funcs {
        if h == hash {
            let cname = CString::new(name).unwrap();
            let addr = GetProcAddress(hmodule, &cname);
            return addr.map_or(null_mut(), |p| p as *mut c_void);
        }
    }

    null_mut()
}
