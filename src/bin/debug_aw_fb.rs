fn main() {
    let rom_data = std::fs::read("dev-roms/another-world.gba").expect("Failed to read ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    for _ in 0..4 {
        gba_emu::emulator::run_frame();
    }
    
    let emu = gba_emu::emulator::get_emu();
    let fb = emu.framebuffer();
    
    eprintln!("Framebuffer[0] = 0x{:08X}", fb[0]);
    eprintln!("  R={} G={} B={} A={}", fb[0] & 0xFF, (fb[0] >> 8) & 0xFF, (fb[0] >> 16) & 0xFF, (fb[0] >> 24) & 0xFF);
    
    // Check first few unique values
    let mut unique = std::collections::HashSet::new();
    for &px in fb.iter() {
        unique.insert(px);
    }
    eprintln!("Unique framebuffer values: {}", unique.len());
    for &v in unique.iter().take(10) {
        eprintln!("  0x{:08X}: R={} G={} B={}", v, v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF);
    }
}
