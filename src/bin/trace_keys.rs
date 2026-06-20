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
    
    // Run 5 frames without input
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Now press Start (bit 3) and run 30 more frames
    gba_emu::emulator::set_keys(0x008); // Start
    for frame in 5..35 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let pc = emu.cpu.r[15];
        if dc != 0x0080 {
            eprintln!("Frame {}: DC=0x{:04X} PC=0x{:08X} - DISPLAY CHANGED!", frame, dc, pc);
        }
    }
    
    // Release keys
    gba_emu::emulator::set_keys(0);
    for frame in 35..60 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let pc = emu.cpu.r[15];
        if dc != 0x0080 {
            eprintln!("Frame {}: DC=0x{:04X} PC=0x{:08X} - DISPLAY CHANGED!", frame, dc, pc);
        }
    }
    
    let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    eprintln!("\nFinal: DC=0x{:04X} PC=0x{:08X}", dc, emu.cpu.r[15]);
    
    // Check oracle with same input
    eprintln!("\nChecking oracle with Start pressed at frame 5...");
}
