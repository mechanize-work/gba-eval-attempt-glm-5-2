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
    
    // Step to 226834 (where game enters VBlankIntrWait loop)
    for i in 0..226834 {
        gba_emu::emulator::step_one();
    }
    
    // Check IWRAM at various offsets
    eprintln!("IWRAM at key offsets:");
    for offset in [0x243C, 0x2504, 0x2400, 0x2480, 0x2580] {
        let addr = 0x03000000 + offset;
        let val = emu.mem.read_word(addr);
        eprintln!("  [0x{:08X}] = 0x{:08X}", addr, val);
    }
    
    // Check what the game's function pointer table looks like
    eprintln!("\nFunction pointer table at 0x0300243C:");
    for i in 0..16 {
        let addr = 0x0300243C + i * 4;
        let val = emu.mem.read_word(addr);
        eprintln!("  [0x{:08X}] = 0x{:08X}", addr, val);
    }
    
    // Check where the game stores DISPCNT value
    // The game should write to 0x04000000 at some point
    // Let me check what ROM address writes to DISPCNT
    eprintln!("\nSearching ROM for DISPCNT writes...");
    // The game uses STR to write to 0x04000000
    // Let's check if there's a MOV + STR sequence targeting 0x04000000
}
