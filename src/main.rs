mod process;

use std::ffi::OsString;
use crate::process::Process;

fn main() {
    let processes = Process::list_readable();
    println!("Found {} readable processes", processes.len());

    // Find the notepad process and throw out the rest
    // Note: if there are multiple instances of notepad running, we will just take one of them
    match processes.into_iter().find(|p| p.get_name().to_str().unwrap().ends_with("Windows\\System32\\notepad.exe")) {
        Some(notepad) => {
            println!("Found notepad");

            // NOTE: 0x2C470 is machine dependent... Could we do something to find the right pointer depending on the
            // machine?

            println!("Base of dll: 0x{:X}", notepad.base_address);
            println!("First pointer: 0x{:X}", notepad.base_address + 0x2C470);

            let buffer: &mut [_] = &mut [0u8; 8];

            notepad.read_memory(buffer, notepad.base_address + 0x2C470);

            let second_pointer = read_ptr(buffer);
            println!("Second pointer: 0x{:X}", second_pointer);

            notepad.read_memory(buffer, second_pointer);
            let third_pointer = read_ptr(buffer);
            println!("Third pointer: 0x{:X}", third_pointer);

            let buffer: &mut [_] = &mut [0u8; 1024];
            notepad.read_memory(buffer, third_pointer);
            let string = read_windows_string(buffer);
            println!("{}", string.to_string_lossy());
        },
        None => {
            println!("Notepad is not running");
        }
    }
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
