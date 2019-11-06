mod process;

use crate::process::Process;

fn main() {
    let processes = Process::list_readable();
    println!("Found {} readable processes", processes.len());

    // Find notepad process and throw out the rest
    match processes.into_iter().find(|p| p.get_name().to_str().unwrap().ends_with("Windows\\System32\\notepad.exe")) {
        Some(notepad) => {
            // Note: if there are multiple instances of notepad running, we will just take one of them
            println!("Found notepad");
        },
        None => {
            println!("Notepad is not running");
        }
    }
}
