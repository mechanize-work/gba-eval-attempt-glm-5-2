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
    
    // Run 1 frame (init)
    gba_emu::emulator::run_frame();
    
    // Check BIOS before CpuFastSet
    eprintln!("BIOS before frame 1:");
    for i in [0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C, 0x128, 0x12C] {
        let val = emu.mem.read_word(0x00000000 + i);
        eprintln!("  BIOS[0x{:04X}] = 0x{:08X}", i, val);
    }
    
    // Check IWRAM source for CpuFastSet
    eprintln!("\nIWRAM source at 0x03007E34:");
    for i in 0..8 {
        let val = emu.mem.read_word(0x03007E34 + i * 4);
        eprintln!("  IWRAM[0x{:04X}] = 0x{:08X}", 0x7E34 + i * 4, val);
    }
    
    // Run frame 1 (should trigger CpuFastSet to BIOS)
    gba_emu::emulator::run_frame();
    
    // Check BIOS after CpuFastSet
    eprintln!("\nBIOS after frame 1:");
    for i in [0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C, 0x128, 0x12C] {
        let val = emu.mem.read_word(0x00000000 + i);
        eprintln!("  BIOS[0x{:04X}] = 0x{:08X}", i, val);
    }
}
