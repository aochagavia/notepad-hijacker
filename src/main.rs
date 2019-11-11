mod process;

use std::ffi::OsString;
use crate::process::Process;

fn main() {
    let processes = Process::list_readable();
    println!("Found {} readable processes", processes.len());

    // Find the notepad process and throw out the rest
    // Note: if there are multiple instances of notepad running, we will just take one of them
    if let Some(notepad) = processes.into_iter().find(|p| p.get_name().to_str().unwrap().ends_with("Windows\\System32\\notepad.exe")) {
        println!("Found notepad");

        // NOTE: 0x2C470 is machine dependent... Could we do something to find the right pointer depending on the
        // machine?

        let string_ptr = chase_ptr(&notepad, notepad.base_address + 0x2C470, 2);

        let buffer: &mut [_] = &mut [0u8; 1024];
        notepad.read_memory(buffer, string_ptr);
        let string = read_windows_string(buffer);
        let string = string.to_string_lossy();
        println!("String length: {} ({} bytes)", string.len(), string.len() * 2);
        println!("Currently open text: {}", string);

        // Replace string by itself, but now reversed
        write_windows_string(&notepad, string.bytes().rev(), string_ptr);
        print!("Text has been reversed!");
    } else {
        println!("Notepad is not running");
    }
}

fn chase_ptr(process: &Process, address: u64, depth: u8) -> u64 {
    assert!(depth > 0);

    let mut buffer = vec![0u8; 8];
    let mut final_value = address;
    for _ in 0..depth {
        process.read_memory(&mut buffer, final_value);
        final_value = read_ptr(&buffer);
    }

    final_value
}

fn read_ptr(bytes: &[u8]) -> u64 {
    assert!(bytes.len() == 8);
    let mut x = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        x += (byte as u64) << (i * 8);
    }
    x
}

fn read_windows_string(bytes: &[u8]) -> OsString {
    use std::os::windows::ffi::OsStringExt;

    // Not sure whether this unsafe is OK, but whatever... this is just an experiment
    assert!(bytes.len() % 2 == 0);
    let string: &[u16] = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() / 2) };

    let length = string.iter().position(|&x| x  == 0).unwrap_or(string.len());
    OsString::from_wide(&string[..length])
}

fn write_windows_string(process: &Process, s: impl Iterator<Item=u8>, address: u64) {
    let bytes: Vec<_> = s.flat_map(|c| vec![c, 0]).collect();
    process.write_memory(&bytes, address);
}
