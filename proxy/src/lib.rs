#![feature(decl_macro)]

use once_cell::sync::OnceCell;
use paste::paste;
use std::ffi::c_void;
use std::mem::transmute;
use std::num::NonZeroIsize;
use std::path::PathBuf;
use std::thread;
use windows_sys::core::PCSTR;
use windows_sys::core::PCWSTR;
use windows_sys::core::PSTR;
use windows_sys::core::PWSTR;
use windows_sys::s;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::Foundation::MAX_PATH;
use windows_sys::Win32::Storage::FileSystem::GET_FILE_VERSION_INFO_FLAGS;
use windows_sys::Win32::Storage::FileSystem::VER_FIND_FILE_FLAGS;
use windows_sys::Win32::Storage::FileSystem::VER_FIND_FILE_STATUS;
use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
use windows_sys::Win32::System::LibraryLoader::LoadLibraryA;
use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryA;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

#[no_mangle]
extern "system" fn DllMain(_: HMODULE, reason: u32, _: usize) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(__inject);
    }

    true
}

fn __inject() {
    // We do this to defer loading of libraries (opengl32, really) until later.
    // Why does this work?? I DON'T KNOW!!
    unsafe { LoadLibraryA(s!("speng_starb.dll")) };
}

// WARNING: SPAGHETTI CODE BELOW

// FIXME: Somehow my old build script doesn't work but this does lol
pub(crate) macro __lazy_export($(fn $f:ident($($i:ident: $a:ty),*) -> $r:ty);+;) {
    #[inline]
    #[must_use]
    pub fn __h_version() -> HMODULE {
        static VERSION: OnceCell<HMODULE> = OnceCell::new();

        *VERSION.get_or_init(|| unsafe {
            // mess...
            let mut buffer = [0u8; MAX_PATH as usize];
            let buffer_len = GetSystemDirectoryA(buffer.as_mut_ptr(), buffer.len() as u32);

            // 0 == failed
            assert_ne!(buffer_len, 0u32);

            let dir =
                PathBuf::from(String::from_utf8(buffer[..buffer_len as usize].to_vec()).unwrap())
                    .join("version.dll");

            let dir = [dir.to_str().unwrap().as_bytes(), &[0u8]].concat();

            // 0 == failed
            NonZeroIsize::new(LoadLibraryA(dir.as_ptr())).unwrap().get()
        })
    }

    paste! {
        $(
            #[allow(clippy::many_single_char_names)]
            #[export_name = "" $f ""]
            unsafe extern "system" fn [<__ $f:snake>]($($i: $a),*) -> $r {
                static [<$f:snake:upper>]: OnceCell<usize> = OnceCell::new();

                unsafe {
                    transmute::<usize, unsafe extern "system" fn($($a),*) -> $r>(
                        *[<$f:snake:upper>].get_or_init(|| {
                            GetProcAddress(
                                __h_version(),
                                format!("{}\0", stringify!($f)).as_ptr(),
                            )
                            .unwrap() as usize
                        }),
                    )($($i),*)
                }
            }
        )*
    }
}

#[rustfmt::skip]
__lazy_export! {
    fn GetFileVersionInfoA(a: PCSTR, b: u32, c: u32, d: *mut c_void) -> i32;
    fn GetFileVersionInfoExA(a: GET_FILE_VERSION_INFO_FLAGS, b: PCSTR, c: u32, d: u32, e: *mut c_void) -> i32;
    fn GetFileVersionInfoExW(a: GET_FILE_VERSION_INFO_FLAGS, b: PCWSTR, c: u32, d: u32, e: *mut c_void) -> i32;
    fn GetFileVersionInfoSizeA(a: PCSTR, b: *mut u32) -> u32;
    fn GetFileVersionInfoSizeExA(a: GET_FILE_VERSION_INFO_FLAGS, b: PCSTR, c: *mut u32) -> u32;
    fn GetFileVersionInfoSizeExW(a: GET_FILE_VERSION_INFO_FLAGS, b: PCWSTR, c: *mut u32) -> u32;
    fn GetFileVersionInfoSizeW(a: PCWSTR, b: *mut u32) -> u32;
    fn GetFileVersionInfoW(a: PCWSTR, b: u32, c: u32, d: *mut c_void) -> i32;
    fn VerFindFileA(a: VER_FIND_FILE_FLAGS, b: PCSTR, c: PCSTR, d: PCSTR, e: PSTR, f: *mut u32, g: PSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerFindFileW(a: VER_FIND_FILE_FLAGS, b: PCWSTR, c: PCWSTR, d: PCWSTR, e: PWSTR, f: *mut u32, g: PWSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerInstallFileA(a: VER_FIND_FILE_FLAGS, b: PCSTR, c: PCSTR, d: PCSTR, e: PSTR, f: PSTR, g: PSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerInstallFileW(a: VER_FIND_FILE_FLAGS, b: PCWSTR, c: PCWSTR, d: PCWSTR, e: PWSTR, f: PWSTR, g: PWSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerLanguageNameA(a: u32, b: PSTR, c: u32) -> u32;
    fn VerLanguageNameW(a: u32, b: PWSTR, c: u32) -> u32;
    fn VerQueryValueA(a: *const c_void, b: PCSTR, c: *mut *mut c_void, d: *mut u32) -> i32;
    fn VerQueryValueW(a: *const c_void, b: PCWSTR, c: *mut *mut c_void, d: *mut u32) -> i32;
}
