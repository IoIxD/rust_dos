cargo build --release &&
cargo objcopy --release -- -O binary --binary-architecture=i386:x86 rust_dos.com &&
dosbox
