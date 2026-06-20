// Test host program that loads the GBA emulator wasm and runs it
// Usage: host <wasm_file> <rom_file> <frames>
use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <wasm_file> <rom_file> <frames> [--dump-frames <dir>]", args[0]);
        std::process::exit(1);
    }

    let wasm_file = &args[1];
    let rom_file = &args[2];
    let frames: u32 = args[3].parse().unwrap();
    
    let dump_dir = if args.len() > 5 && args[4] == "--dump-frames" {
        Some(args[5].clone())
    } else {
        None
    };

    // Load wasm
    let wasm_bytes = fs::read(wasm_file).expect("Failed to read wasm file");
    let rom_bytes = fs::read(rom_file).expect("Failed to read rom file");

    println!("ROM size: {} bytes", rom_bytes.len());

    // Use wasmtime to run
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &wasm_bytes).expect("Failed to compile wasm");
    
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[]).expect("Failed to instantiate wasm");

    let get_func = |store: &wasmtime::Store<()>, name: &str| -> wasmtime::Func {
        instance.get_func(store, name).expect(&format!("Missing export: {}", name))
    };

    // Call emu_init
    let init_func = get_func(&store, "emu_init");
    let init_result = init_func.call(&mut store, &[]).expect("emu_init failed");
    println!("emu_init returned: {:?}", init_result[0].i32());

    // Get rom buffer pointer
    let rom_buf_func = get_func(&store, "emu_rom_buffer");
    let rom_buf_result = rom_buf_func.call(&mut store, &[]).expect("emu_rom_buffer failed");
    let rom_buf_ptr = rom_buf_result[0].i32().unwrap() as u32;
    println!("ROM buffer at: 0x{:08X}", rom_buf_ptr);

    // Write ROM data into wasm memory
    let memory = instance.get_memory(&store, "memory").expect("No memory export");
    let rom_len = rom_bytes.len() as i32;
    for (i, byte) in rom_bytes.iter().enumerate() {
        memory.data_mut(&mut store)[(rom_buf_ptr as usize) + i] = *byte;
    }
    println!("Wrote {} bytes to ROM buffer", rom_bytes.len());

    // Call emu_load_rom
    let load_func = get_func(&store, "emu_load_rom");
    let load_result = load_func.call(&mut store, &[rom_len.into()]).expect("emu_load_rom failed");
    println!("emu_load_rom returned: {:?}", load_result[0].i32());

    // Run frames
    let run_frame_func = get_func(&store, "emu_run_frame");
    let fb_func = get_func(&store, "emu_framebuffer");
    let audio_buf_func = get_func(&store, "emu_audio_buffer");
    let audio_samples_func = get_func(&store, "emu_audio_samples");
    let audio_rate_func = get_func(&store, "emu_audio_rate");

    let audio_rate = audio_rate_func.call(&mut store, &[]).expect("audio_rate failed");
    println!("Audio rate: {:?}", audio_rate[0].i32());

    for frame in 0..frames {
        run_frame_func.call(&mut store, &[]).expect("emu_run_frame failed");
        
        if let Some(ref dir) = dump_dir {
            // Get framebuffer
            let fb_result = fb_func.call(&mut store, &[]).expect("emu_framebuffer failed");
            let fb_ptr = fb_result[0].i32().unwrap() as usize;
            
            // Read framebuffer (240*160*4 bytes)
            let fb_size = 240 * 160 * 4;
            let fb_data = &memory.data(&store)[fb_ptr..fb_ptr + fb_size];
            
            // Write as PPM
            let ppm_path = format!("{}/frame_{:05}.ppm", dir, frame);
            let mut file = fs::File::create(&ppm_path).expect("Failed to create frame file");
            write!(file, "P6\n240 160\n255\n").unwrap();
            for i in 0..(240 * 160) {
                let r = fb_data[i * 4];
                let g = fb_data[i * 4 + 1];
                let b = fb_data[i * 4 + 2];
                file.write_all(&[r, g, b]).unwrap();
            }
        }

        // Get audio samples
        let samples_result = audio_samples_func.call(&mut store, &[]).expect("emu_audio_samples failed");
        let samples = samples_result[0].i32().unwrap();
        if frame == 0 {
            println!("Frame 0 audio samples: {}", samples);
        }
    }

    println!("Done running {} frames", frames);
}
