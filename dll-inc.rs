use std::ffi::CStr;
use windows_sys::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Diagnostics::ToolHelp::*,
    Win32::System::Diagnostics::Debug::*,
    Win32::System::Threading::*,
    Win32::System::Memory::*,
    Win32::System::LibraryLoader::*,
};

fn find_process_id(target_name: &str) -> Result<u32, &'static str> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return Err("snapshot failed");
        }

        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let mut has_entry = Process32First(snap, &mut entry) == 1;
        while has_entry {
            let exe_name = CStr::from_ptr(entry.szExeFile.as_ptr().cast())
                .to_str()
                .unwrap_or("");
            if exe_name.eq_ignore_ascii_case(target_name) {
                return Ok(entry.th32ProcessID);
            }
            has_entry = Process32Next(snap, &mut entry) == 1;
        }
    }
    Err("not found")
}

// Sama Inekcia
fn load_library_into(pid_name: &str, library_path: &str) -> Result<(), &'static str> {
    let pid = find_process_id(pid_name)?;

    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        if handle == INVALID_HANDLE_VALUE {
            return Err("Cannot open process");
        }

        let size = library_path.len() + 1;

        let remote_mem = VirtualAllocEx(
            handle,
            std::ptr::null(),
            size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE,
        );

        if remote_mem.is_null() {
            CloseHandle(handle);
            return Err("Memory allocation failed");
        }

        let mut written = 0;
        let write_ok = WriteProcessMemory(
            handle,
            remote_mem,
            library_path.as_ptr().cast(),
            size,
            &mut written,
        );

        if write_ok == 0 {
            CloseHandle(handle);
            return Err("WriteProcessMemory failed");
        }

        let kernel = GetModuleHandleA(s!("kernel32.dll"));
        if kernel == 0 {
            CloseHandle(handle);
            return Err("kernel32 not found");
        }
        let load_addr = GetProcAddress(kernel, s!("LoadLibraryA"));
        if load_addr.is_none() {
            CloseHandle(handle);
            return Err("LoadLibraryA not found");
        }

        let start_routine: LPTHREAD_START_ROUTINE =
            std::mem::transmute(load_addr.unwrap());
        let mut thread_id = 0;
        let thread_handle = CreateRemoteThread(
            handle,
            std::ptr::null(),
            0,
            start_routine,
            remote_mem,
            0,
            &mut thread_id,
        );

        if thread_handle == 0 {
            CloseHandle(handle);
            return Err("Thread creation failed");
        }

        WaitForSingleObject(thread_handle, INFINITE);
        CloseHandle(thread_handle);
        CloseHandle(handle);
    }

    Ok(())
}

fn main() {
    let target_process = ""; // maybe "notepad.exe"
    let dll_to_inject = "";  // path to dLL

    match load_library_into(target_process, dll_to_inject) {
        Ok(_) => println!("Injection completed"),
        Err(e) => println!("Error: {}", e),
    }
}
