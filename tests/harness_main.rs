// Native test harness - runs the emulator directly as a library
// and compares output with oracle reference frames
use std::env;
use std::fs;
use std::io::Write;
use std::process;

// We need to conditionally compile for native target
// When building as native binary, use std

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <rom_file> <frames> [--dump-frames <dir>] [--keys <keys>]", args[0]);
        process::exit(1);
    }

    let rom_file = &args[1];
    let frames: u32 = args[2].parse().unwrap();
    
    let mut dump_dir: Option<String> = None;
    let mut keys: u32 = 0;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--dump-frames" => {
                dump_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--keys" => {
                keys = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            _ => { i += 1; }
        }
    }

    // Read ROM
    let rom_data = fs::read(rom_file).expect("Failed to read ROM");
    eprintln!("ROM size: {} bytes", rom_data.len());

    // We need to call the emulator functions.
    // Since we're building natively, we can use the rlib crate.
    // But the emulator uses no_std, so we need to adapt.
    
    // Actually, let's use a different approach - build the test harness
    // as part of the cargo project with a binary target.
    
    eprintln!("This binary needs to be compiled with the gba_emu crate.");
    eprintln!("Use: cargo test --test harness");
    process::exit(1);
}
