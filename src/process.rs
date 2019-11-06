use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::mem;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::CloseHandle;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::psapi;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt;

pub struct Process {
    handle: *mut c_void,
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

impl Process {
    pub fn new(pid: u32) -> Result<Process, u32> {
        let handle = unsafe {
            OpenProcess(
                winnt::PROCESS_QUERY_INFORMATION | winnt::PROCESS_VM_READ,
                FALSE,
                pid,
            )
        };

        if handle == NULL {
            Err(unsafe { GetLastError() })
        } else {
            Ok(Process { handle })
        }
    }

    pub fn list_readable() -> Vec<Process> {
        list_all_pids()
            .into_iter()
            .filter(|&pid| pid != 0) // Filter out the System Idle Process
            .flat_map(|pid| Process::new(pid))
            .collect()
    }

    pub fn get_name(&self) -> OsString {
        let mut buffer = vec![0u16; 1024];
        let bytes_read = unsafe {
            let buffer_len = (buffer.len() * mem::size_of::<u16>()) as u32;
            psapi::GetProcessImageFileNameW(self.handle, buffer.as_mut_ptr(), buffer_len)
        };

        if bytes_read == 0 {
            // See https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--500-999-
            panic!("Error getting process name! Error code: {}", unsafe { GetLastError() });
        }

        buffer.truncate(bytes_read as usize);

        OsString::from_wide(&buffer)
    }
}

fn list_all_pids() -> Vec<u32> {
    let mut buffer = vec![0u32; 1024];
    let mut bytes_written = 0u32;
    let success = unsafe {
        let buffer_len = (buffer.len() * mem::size_of::<DWORD>()) as u32;
        psapi::EnumProcesses(buffer.as_mut_ptr(), buffer_len, &mut bytes_written as *mut _)
    };

    if success == 0 {
        // See https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--500-999-
        panic!("Error listing processes! Error code: {}", unsafe { GetLastError() });
    }

    let processes_found = bytes_written as usize / mem::size_of::<DWORD>();
    if processes_found == buffer.len() {
        println!("Warning: the buffer was completely filled... There could well be more processes!");
    }

    buffer.truncate(processes_found);
    buffer
}
