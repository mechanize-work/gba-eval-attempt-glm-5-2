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
    
    // Run 1 frame to get to VBlankIntrWait
    gba_emu::emulator::run_frame();
    
    eprintln!("After frame 0:");
    eprintln!("  PC=0x{:08X} halted={}", emu.cpu.r[15], emu.cpu.halted);
    eprintln!("  scanline={} cycle_in_scanline={}", emu.current_scanline, emu.cycle_in_scanline);
    eprintln!("  vblank_waiting={} vblank_occurred={}", emu.vblank_waiting, emu.vblank_occurred);
    eprintln!("  cycle_count={}", emu.cycle_count);
    
    // Run frame 1
    gba_emu::emulator::run_frame();
    eprintln!("\nAfter frame 1:");
    eprintln!("  PC=0x{:08X} halted={}", emu.cpu.r[15], emu.cpu.halted);
    eprintln!("  scanline={} cycle_in_scanline={}", emu.current_scanline, emu.cycle_in_scanline);
    eprintln!("  vblank_waiting={} vblank_occurred={}", emu.vblank_waiting, emu.vblank_occurred);
}
