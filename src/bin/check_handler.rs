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
    
    // Run 2 frames to reach VBlankIntrWait
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    // Check IWRAM vector table
    eprintln!("IWRAM vector table at 0x03000000:");
    for i in 0..16 {
        let addr = 0x03000000 + i * 4;
        let val = emu.mem.read_word(addr);
        eprintln!("  [0x{:08X}] = 0x{:08X}", addr, val);
    }
    
    // Check handler at 0x03007FFC
    let handler = emu.mem.read_word(0x03007FFC);
    eprintln!("\n[0x03007FFC] = 0x{:08X}", handler);
    
    // Check what the BIOS copied to IWRAM
    eprintln!("\nIWRAM[0x03000000] first 32 bytes (as halfwords):");
    for i in 0..16 {
        let addr = 0x03000000 + i * 2;
        let val = emu.mem.read_half(addr);
        eprintln!("  [0x{:08X}] = 0x{:04X}", addr, val);
    }
}
