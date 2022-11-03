pub mod sel_object;

use std::{collections::HashMap, mem};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use windows::Win32::{
    Foundation::{self, HANDLE, HINSTANCE},
    System::{Diagnostics::Debug, ProcessStatus, Threading},
};

pub struct Handler {
    pub handle: HANDLE,
    pub modules: HashMap<String, usize>,
}

impl Handler {
    pub fn new(sys: &mut System) -> Self {
        sys.refresh_processes();

        let handle = match unsafe {
            Threading::OpenProcess(
                Threading::PROCESS_QUERY_INFORMATION
                    | Threading::PROCESS_VM_OPERATION
                    | Threading::PROCESS_VM_READ
                    | Threading::PROCESS_VM_WRITE,
                false,
                match sys.processes_by_exact_name("SpaceEngine.exe").nth(0usize) {
                    Some(pc) => pc.pid().as_u32(),
                    None => todo!(),
                },
            )
        } {
            Ok(ph) => ph,
            Err(_) => todo!(),
        };

        let modules = {
            let (addresses, num_of) = {
                // I'm fairly sure initializing as an array instead of a vector is
                // cheaper, plus we already collect it as a Vec<usize> anyway
                let addresses = [0usize; 256usize];
                let mut num_of = 0u32;

                unsafe {
                    ProcessStatus::K32EnumProcessModules(
                        handle,
                        addresses.as_ptr() as _,
                        mem::size_of::<[usize; 256usize]>() as _,
                        &mut num_of,
                    );
                }

                (
                    addresses
                        // Iterate over addresses
                        .iter()
                        // Filter out invalid addresses, then deref
                        .filter_map(|x| (*x != 0usize).then(|| *x))
                        // Collect
                        .collect::<Vec<usize>>(),
                    num_of as usize / mem::size_of::<usize>(),
                )
            };

            if addresses.len() != num_of {
                todo!();
            }

            let mut names = Vec::<String>::with_capacity(num_of);

            for address in addresses.iter() {
                let mut raw_name = [0u16; Foundation::MAX_PATH as _];

                unsafe {
                    ProcessStatus::K32GetModuleBaseNameW(
                        handle,
                        HINSTANCE(*address as _),
                        &mut raw_name,
                    );
                }

                names.push(String::from_utf16_lossy(&raw_name).replace('\0', ""))
            }

            // todo!(); I should refactor this a bit.

            let mut map = HashMap::new();

            for (name, address) in names.iter().zip(addresses) {
                map.insert(name.clone(), address);
            }

            map
        };

        Self { handle, modules }
    }

    pub fn close(&self) {
        unsafe { Foundation::CloseHandle(self.handle) };
    }

    // todo!(); make everything not "safe" here return Result<T>

    pub fn flush(&self, address: usize, size: usize) {
        unsafe {
            Debug::FlushInstructionCache(self.handle, Some(address as _), size);
        }
    }

    pub fn read(&self, address: usize, size: usize) -> Vec<u8> {
        let bytes = vec![0u8; size];
        let mut bytes_read = 0usize;

        match unsafe {
            Debug::ReadProcessMemory(
                self.handle,
                address as _,
                bytes.as_ptr() as _,
                bytes.len(),
                Some(&mut bytes_read),
            )
            .as_bool()
        } {
            true => {
                if bytes.len() != bytes_read {
                    todo!();
                }

                bytes
            }
            false => todo!(),
        }
    }

    pub fn write(&self, address: usize, bytes: Vec<u8>) {
        let mut bytes_written = 0usize;

        match unsafe {
            Debug::WriteProcessMemory(
                self.handle,
                address as _,
                bytes.as_ptr() as _,
                bytes.len(),
                Some(&mut bytes_written),
            )
            .as_bool()
        } {
            true => {}
            false => {}
        }

        self.flush(address, bytes.len());
    }

    pub fn base(&self) -> usize {
        self.base_of("SpaceEngine.exe")
    }

    pub fn base_of(&self, module: &str) -> usize {
        match self.modules.get(module) {
            Some(ba) => *ba,
            None => todo!(),
        }
    }
}
