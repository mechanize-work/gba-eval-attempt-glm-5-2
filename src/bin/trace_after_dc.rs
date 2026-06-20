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
    
    // Step to 226439 (DISPCNT change)
    for i in 0..226439 { gba_emu::emulator::step_one(); }
    
    eprintln!("At DISPCNT change: DC=0x{:04X} PC=0x{:08X}",
        (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8),
        emu.cpu.r[15]);
    
    // Run 60 more frames
    for f in 0..60 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if f < 5 || f % 10 == 0 || dc != 0x1000 {
            eprintln!("Frame {}: DC=0x{:04X} halted={} PC=0x{:08X}",
                f, dc, emu.cpu.halted, emu.cpu.r[15]);
        }
        
        let fb = &emu.ppu.framebuffer;
        let non_black = fb.iter().filter(|p| **p != 0 && **p != 0xFF000000).count();
        if non_black > 0 {
            eprintln!("  Non-black pixels: {}", non_black);
        }
    }
}
