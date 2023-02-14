#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

mod exe;

use crate::exe::EXE;

// Linux users: <https://gist.github.com/michaelbutler/f364276f4030c5f449252f2c4d960bd2>
#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("`Star Browser Utilities` should only be compiled on `Windows`");

fn main() {
    starb_logging::init();

    EXE.init("SpaceEngine.exe").unwrap();
    EXE.write_to(100000000000, u32::MAX).unwrap();
    EXE.read_to::<u32>(1).unwrap();
    EXE.write(1, 1).unwrap();
}
