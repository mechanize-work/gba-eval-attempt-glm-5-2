use std::fs;
fn main() {
    let rom_data = fs::read("dev-roms/anguna.gba").expect("Failed to read ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    let emu = gba_emu::emulator::get_emu();
    
    // Run enough frames to get past init (need ~2 frames of init + VBlank cycles)
    // Step until DISPCNT changes
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc == 0x1000 {
            eprintln!("DISPCNT=0x1000 at step {}", i);
            break;
        }
    }
    
    // Now run 10 more frames to get the display output
    for f in 0..10 {
        gba_emu::emulator::run_frame();
    }
    
    // Dump framebuffer as PPM
    let fb = &emu.ppu.framebuffer;
    let mut file = fs::File::create("/tmp/emu_frame.ppm").expect("Failed to create file");
    use std::io::Write;
    write!(file, "P6\n240 160\n255\n").unwrap();
    for i in 0..(240*160) {
        let p = fb[i];
        let r = (p & 0xFF) as u8;
        let g = ((p >> 8) & 0xFF) as u8;
        let b = ((p >> 16) & 0xFF) as u8;
        file.write_all(&[r, g, b]).unwrap();
    }
    
    eprintln!("Framebuffer dumped to /tmp/emu_frame.ppm");
    eprintln!("DISPCNT=0x{:04X}", (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
    
    // Count colors
    let mut colors = std::collections::HashSet::new();
    for i in 0..(240*160) {
        colors.insert(fb[i]);
    }
    eprintln!("Unique colors: {}", colors.len());
}
