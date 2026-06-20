// Test harness - runs the emulator natively and outputs frames for comparison
use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <rom_file> <frames> [--dump-frames <dir>] [--keys <keys>]", args[0]);
        std::process::exit(1);
    }

    let rom_file = &args[1];
    let frames: u32 = args[2].parse().unwrap();
    
    let mut dump_dir: Option<String> = None;
    let mut keys: u32 = 0;
    let mut key_frame: u32 = 0;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--dump-frames" => {
                dump_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--keys" => {
                keys = args[i + 1].parse().unwrap_or(0);
                key_frame = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(0);
                i += 3;
            }
            _ => { i += 1; }
        }
    }

    // Read ROM
    let rom_data = fs::read(rom_file).expect("Failed to read ROM");
    eprintln!("ROM size: {} bytes", rom_data.len());

    // Initialize emulator
    gba_emu::emulator::init();
    
    // Get ROM buffer and copy ROM data
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    
    // Load ROM
    let result = gba_emu::emulator::load_rom(rom_data.len());
    eprintln!("emu_load_rom returned: {}", result);

    // Run frames
    for frame in 0..frames {
        if frame == key_frame && keys != 0 {
            gba_emu::emulator::set_keys(keys);
        }
        
        gba_emu::emulator::run_frame();
        
        if let Some(ref dir) = dump_dir {
            // Get framebuffer and write as PPM
            let fb_ptr = gba_emu::emulator::framebuffer_ptr();
            let fb_size = 240 * 160;
            
            let ppm_path = format!("{}/frame_{:05}.ppm", dir, frame);
            let mut file = fs::File::create(&ppm_path).expect("Failed to create frame file");
            write!(file, "P6\n240 160\n255\n").unwrap();
            
            unsafe {
                let fb = std::slice::from_raw_parts(fb_ptr, fb_size);
                for pixel in fb {
                    // ABGR format: 0xAABBGGRR
                    let r = (*pixel & 0xFF) as u8;
                    let g = ((*pixel >> 8) & 0xFF) as u8;
                    let b = ((*pixel >> 16) & 0xFF) as u8;
                    file.write_all(&[r, g, b]).unwrap();
                }
            }
        }
    }

    // Print audio info
    let audio_samples = gba_emu::emulator::audio_samples();
    let audio_rate = gba_emu::emulator::audio_rate();
    eprintln!("Audio: {} samples at {} Hz", audio_samples, audio_rate);
    
    eprintln!("Done running {} frames", frames);
}
