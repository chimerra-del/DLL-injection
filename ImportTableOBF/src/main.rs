mod api;
use api::resolve_api;

const fn ror32(value: u32, bits: u32) -> u32 {
    (value >> bits) | (value << (32 - bits))
}

pub const fn hash_api(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut hash = 0u32;
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i];
        // uppercase
        let c = if c >= b'a' && c <= b'z' {
            c - 32
        } else {
            c
        };
        hash = ror32(hash, 13);
        hash = hash.wrapping_add(c as u32);
        i += 1;
    }
    hash
}

fn find_process_id(target_name: &str) -> Result<u32, &'static str> {
    unsafe {
        let snapshot_addr = resolve_api(hash_api("CreateToolhelp32Snapshot"));
        if snapshot_addr == 0 {
            return Err("CreateToolhelp32Snapshot not found");
        }
        let first_addr = resolve_api(hash_api("Process32FirstW"));
        if first_addr == 0 {
            return Err("Process32FirstW not found");
        }
        let next_addr = resolve_api(hash_api("Process32NextW"));
        if next_addr == 0 {
            return Err("Process32NextW not found");
        }
        let snapshot_fn: CreateToolhelp32SnapshotFn =
            std::mem::transmute(snapshot_addr);
        let first_fn: Process32FirstFn =
            std::mem::transmute(first_addr);
        let next_fn: Process32NextFn =
            std::mem::transmute(next_addr);
        let snap = snapshot_fn(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return Err("snapshot failed");
        }
        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        let mut has_entry = first_fn(snap, &mut entry) != 0;
        while has_entry {
            let exe_name = std::ffi::CStr::from_ptr(entry.szExeFile.as_ptr().cast())
                .to_str()
                .unwrap_or("");

            if exe_name.eq_ignore_ascii_case(target_name) {
                return Ok(entry.th32ProcessID);
            }
            has_entry = next_fn(snap, &mut entry) != 0;
        }
    }

    Err("not found")
}
