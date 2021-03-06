use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::{mem, ptr};

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, LPVOID, LPCVOID, FALSE, HMODULE};
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::CloseHandle;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::psapi;
use winapi::um::psapi::{EnumProcessModules, GetModuleInformation, MODULEINFO};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt;

pub struct Process {
    handle: *mut c_void,
    pub pid: u32,
    pub base_address: u64,
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
                winnt::PROCESS_QUERY_INFORMATION | winnt::PROCESS_VM_READ | winnt::PROCESS_VM_WRITE,
                FALSE,
                pid,
            )
        };


        if handle == NULL {
            Err(unsafe { GetLastError() })
        } else {
            let base_address = get_base_address(handle);
            Ok(Process { handle, pid, base_address })
        }
    }

    pub fn read_memory(&self, buffer: &mut [u8], addr: u64) {
        let success = unsafe {
            ReadProcessMemory(
                self.handle,
                addr as LPVOID,
                buffer.as_mut_ptr() as LPVOID,
                buffer.len(),
                ptr::null_mut()
            )
        };

        if success == 0 {
            panic!("Error reading bytes from process! Error code: {}", unsafe { GetLastError() });
        }
    }

    pub fn write_memory(&self, buffer: &[u8], addr: u64) {
        let success = unsafe {
            WriteProcessMemory(
                self.handle,
                addr as LPVOID,
                buffer.as_ptr() as LPCVOID,
                buffer.len(),
                ptr::null_mut()
            )
        };

        if success == 0 {
            panic!("Error writing bytes to process! Error code: {}", unsafe { GetLastError() });
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

fn get_base_address(handle: *mut c_void) -> u64 {
    let mut module_handles: Vec<HMODULE> = vec![ptr::null_mut(); 1];
    let mut bytes_written = 0u32;
    let success = unsafe {
        EnumProcessModules(
            handle,
            module_handles.as_mut_ptr(),
            (module_handles.len() * mem::size_of::<HMODULE>()) as u32,
            &mut bytes_written as *mut _,
        )
    };

    if success == 0 {
        panic!("Error getting module handles from process! Error code: {}", unsafe { GetLastError() });
    }

    if bytes_written == 0 {
        panic!("No module handles retrieved!")
    }

    let module_handle = module_handles[0];

    let mut module_info = MODULEINFO {
        lpBaseOfDll: ptr::null_mut(),
        SizeOfImage: 0,
        EntryPoint: ptr::null_mut(),
    };

    let success = unsafe {
        GetModuleInformation(
            handle,
            module_handle,
            &mut module_info as *mut _,
            mem::size_of_val(&module_info) as u32,
        )
    };

    if success == 0 {
        panic!("Error getting module info from process! Error code: {}", unsafe { GetLastError() });
    }

    module_info.lpBaseOfDll as u64
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
