use eyre::Result;
use once_cell::sync::OnceCell;
use path_clean::PathClean;
use region::protect_with_handle;
use region::Protection;
use windows_sys::Win32::System::ProcessStatus::EnumProcessModules;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use std::env::current_exe;
use std::mem::size_of;
use std::path::PathBuf;
use tracing::trace;

/// Base address of SE module
///
/// This is cached, don't worry about calling this multiple times!
///
/// # Panics
///
/// Panics if getting the base address of SE fails.
#[must_use]
pub fn base() -> usize {
    static BASE: OnceCell<usize> = OnceCell::new();

    *BASE.get_or_init(|| {
        let mut base = [0usize; 1024usize];
        let mut needed = 0u32;

        unsafe {
            assert_ne!(
                EnumProcessModules(
                    GetCurrentProcess(),
                    base.as_mut_ptr().cast(),
                    size_of::<[usize; 1024usize]>() as u32,
                    &mut needed,
                ),
                0i32,
                "This failed for some ungodly reason",
            );
        }

        base[0usize]
    })
}

/// Get SE's system folder.
pub fn sys_folder() -> Result<PathBuf> {
    Ok(current_exe()?.join("../").clean())
}

/// Change `p`'s memory protection and write to it, then revert the protection.
///
/// This is common in plugins, so it is here.
///
/// # Safety
///
/// * `p` must point to mapped memory.
/// * The caller must uphold writing to `p` will maintain memory safety.
pub unsafe fn write<T>(p: *mut T, val: T) -> Result<()> {
    trace!("Writing to {p:?}");

    let _guard = unsafe {
        protect_with_handle(
            p.cast_const(),
            size_of::<T>(),
            Protection::READ_WRITE_EXECUTE,
        )?
    };

    unsafe { p.write(val) };
    Ok(())
}

/// Change `p`'s memory protection and read it, then revert the protection.
///
/// This is common in plugins, so it is here.
///
/// # Safety
///
/// * `p` must point to mapped memory.
/// * The caller must uphold writing to `p` will maintain memory safety.
pub unsafe fn read<T>(p: *const T) -> Result<T> {
    trace!("Reading {p:?}");

    let _guard = unsafe { protect_with_handle(p, size_of::<T>(), Protection::READ_EXECUTE)? };

    Ok(unsafe { p.read() })
}
