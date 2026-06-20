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
    
    eprintln!("After frame 0: PC=0x{:08X} halted={} scanline={} vb_occ={}",
        emu.cpu.r[15], emu.cpu.halted, emu.current_scanline, emu.vblank_occurred);
    
    // Run 1 more frame and check
    gba_emu::emulator::run_frame();
    eprintln!("After frame 1: PC=0x{:08X} halted={} scanline={} vb_occ={}",
        emu.cpu.r[15], emu.cpu.halted, emu.current_scanline, emu.vblank_occurred);
    
    // Run 5 more
    for i in 0..5 {
        gba_emu::emulator::run_frame();
        eprintln!("After frame {}: PC=0x{:08X} halted={} scanline={} vb_occ={}",
            i+2, emu.cpu.r[15], emu.cpu.halted, emu.current_scanline, emu.vblank_occurred);
    }
}
