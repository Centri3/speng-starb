#[macro_use]
extern crate tracing;

mod exe;

use crate::exe::EXE;

// Linux users: <https://gist.github.com/michaelbutler/f364276f4030c5f449252f2c4d960bd2>
#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("`Star Browser Utilities` should only be compiled on `Windows`");

fn main() {
    EXE.init("SpaceEngine.exe").unwrap();

    use std::io::Write;
    let mut lol = std::fs::File::create("headers.txt").unwrap();
    writeln!(lol, "{:#x?}", EXE.headers()).unwrap();
    // EXE.save("SpaceEngine.exe").unwrap();
}
