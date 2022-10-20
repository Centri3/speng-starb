use std::{collections::HashMap, mem};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use windows::Win32::{
    Foundation::{self, HANDLE, HINSTANCE},
    System::{ProcessStatus, Threading},
};

pub struct Handler {
    pub handle: HANDLE,
    modules: HashMap<&'static str, usize>,
}

impl Handler {
    pub fn new(sys: &mut System) -> Self {
        sys.refresh_processes();

        let pid = match sys.processes_by_exact_name("SpaceEngine.exe").nth(0usize) {
            Some(pc) => pc.pid().as_u32(),
            None => todo!(),
        };

        let handle = match unsafe {
            Threading::OpenProcess(
                Threading::PROCESS_QUERY_INFORMATION
                    | Threading::PROCESS_VM_OPERATION
                    | Threading::PROCESS_VM_READ
                    | Threading::PROCESS_VM_WRITE,
                false,
                pid,
            )
        } {
            Ok(ph) => ph,
            Err(_) => todo!(),
        };

        let (addresses, num_of) = {
            let mut addresses = vec![0usize; 1024usize];
            let mut num_of = 0u32;

            unsafe {
                ProcessStatus::K32EnumProcessModules(
                    handle,
                    addresses.as_ptr() as _,
                    mem::size_of::<[usize; 1024usize]>() as _,
                    &mut num_of,
                )
            };

            num_of /= mem::size_of::<usize>() as u32;
            addresses.truncate(num_of as _);

            (addresses, num_of as usize)
        };

        let names = {
            let mut names = vec![[0u16; Foundation::MAX_PATH as _]; num_of];

            for (address, i) in addresses.iter().zip(0usize..num_of) {
                unsafe {
                    ProcessStatus::K32GetModuleBaseNameW(
                        handle,
                        HINSTANCE(addresses[i] as _),
                        &mut names[i],
                    )
                };
            }

            names
                .iter()
                .map(|n| String::from_utf16_lossy(n).replace('\0', ""))
                .collect::<Vec<String>>()
        };

        let mut modules = HashMap::new();

        for (name, address) in names.iter().zip(addresses.iter()) {
            modules.insert(name.clone(), address.clone());
        }

        println!("{:x?}", modules.get("SpaceEngine.exe").unwrap());

        Self {
            handle,
            modules: HashMap::new(),
        }
    }

    pub fn close(&self) {
        unsafe { Foundation::CloseHandle(self.handle) };
    }

    pub fn get_base(&self) -> usize {
        self.get_base_of("SpaceEngine.exe")
    }

    pub fn get_base_of(&self, lib: &str) -> usize {
        todo!();
    }

    pub fn still_open(&self, sys: &mut System) -> bool {
        sys.refresh_processes();

        sys.processes_by_exact_name("SpaceEngine.exe")
            .nth(0usize)
            .is_some()
    }
}
