mod process;

use crate::process::Process;

fn main() {
    let processes = Process::list_readable();
    println!("Found {} readable processes", processes.len());

    // Find the notepad process and throw out the rest
    // Note: if there are multiple instances of notepad running, we will just take one of them
    match processes.into_iter().find(|p| p.get_name().to_str().unwrap().ends_with("Windows\\System32\\notepad.exe")) {
        Some(notepad) => {
            println!("Found notepad");
            // Note: this is a pointer to a pointer! It will not return the first letter, but a pointer to it
            let pointer_to_text_buffer = 0x0002C470; // Found using CheatEngine (see https://www.youtube.com/watch?v=nQ2F2iW80Fk)
            println!("First letter: {}", notepad.read_byte_at_relative_address(0));
        },
        None => {
            println!("Notepad is not running");
        }
    }
}
