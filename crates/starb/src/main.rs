#[macro_use]
extern crate tracing;

mod exe;

use crate::exe::EXE;
use bytemuck::Pod;
use parking_lot::RwLock;
use std::fs;
use std::io::Write;
use windows::w;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::System::Threading::OpenThread;
use windows::Win32::System::Threading::SuspendThread;
use windows::Win32::System::Threading::THREAD_SUSPEND_RESUME;
use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
use windows::Win32::UI::WindowsAndMessaging::IDOK;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION;
use windows::Win32::UI::WindowsAndMessaging::MB_OKCANCEL;

// Linux users: <https://gist.github.com/michaelbutler/f364276f4030c5f449252f2c4d960bd2>
#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("`Star Browser Utilities` should only be compiled on `Windows`");

fn read<T: Pod>(bytes: &[u8], start: usize) -> T {
    let size = std::mem::size_of::<T>();
    let mut result = vec![];

    for i in 0usize..size {
        result.push(
            *bytes
                .get(start + i)
                .unwrap_or_else(|| panic!("wtf: {start:x} {i:x}")),
        );
    }

    *bytemuck::from_bytes::<T>(&result)
}

fn main() {
    starb_logging::init();

    let bytes = fs::read(
        "C:/Program Files (x86)/Steam/steamapps/common/SpaceEngine/system/SpaceEngine.exe",
    )
    .unwrap();

    let entry = read::<u32>(&bytes, 0x198);

    for i in 0usize.. {
        let name = String::from_utf8_lossy(&read::<[u8; 8usize]>(&bytes, 0x278 + i * 0x28usize))
            .replace('\0', "");

        if name.is_empty() {
            break;
        } else if name == ".rdata" {
            let r_data = read::<u32>(&bytes, 0x278 + i * 0x3cusize) as usize;

            let mut file = std::fs::File::create("test").unwrap();

            // TODO: Use SizeOfRawData here
            for i in 0usize..100000usize {
                let import = read::<[u32; 2usize]>(&bytes, r_data + i * 8usize);

                if import[1usize] != 0u32 || import[0usize] == 0u32 {
                } else {
                    let address = import[0usize] as usize + 0x2;
                    let mut name = String::new();
                        for offset in address.. {
                            // FIXME: 😳
                            let byte = read::<u8>(&bytes, offset - 0xC00);

                            if byte == 0u8 {
                                break;
                            }

                            name.push(char::from(byte));
                        }

                        writeln!(file, "{}: {:x}", name, import[0]).unwrap();
                }
            }
        }
    }

    /*
    unsafe {
        SuspendThread(OpenThread(THREAD_SUSPEND_RESUME, false, GetCurrentThreadId()).unwrap());
    }
    */

    /*
    unsafe {
        let x = MessageBoxW(
            HWND(0isize),
            w!(
                "Press OK if you want to start SpaceEngine with mods. Please open the Star \
                 Browser Utilities client after doing this, otherwise SpaceEngine will remain \
                 halted. This dialog box will not be shown again.\n\nPress Cancel to abort. You \
                 can unpatch SpaceEngine with the client."
            ),
            w!("Star Browser Utilities"),
            MB_OKCANCEL | MB_ICONINFORMATION,
        );

        match x {
            IDOK => println!("Starting"),
            _ => println!("Aborting"),
        }
    }
    */
}
