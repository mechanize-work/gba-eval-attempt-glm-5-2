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
    
    eprintln!("After frame 0: halted={} scanline={} vb_occ={}",
        emu.cpu.halted, emu.current_scanline, emu.vblank_occurred);
    
    // Now manually step through frame 1, tracking scanline changes
    let mut last_scan = emu.current_scanline;
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        if emu.current_scanline != last_scan {
            if emu.current_scanline == 160 {
                eprintln!("[{}] VBlank! scanline=160 vb_occ={} halted={}",
                    i, emu.vblank_occurred, emu.cpu.halted);
            }
            last_scan = emu.current_scanline;
        }
        
        if !emu.cpu.halted && emu.cpu.r[15] != 0x08008B20 {
            eprintln!("[{}] CPU woke up! PC=0x{:08X} scanline={} vb_occ={}",
                i, emu.cpu.r[15], emu.current_scanline, emu.vblank_occurred);
            break;
        }
        
        if i % 50000 == 0 && i > 0 {
            eprintln!("[{}] Still halted. scanline={} vb_occ={}",
                i, emu.current_scanline, emu.vblank_occurred);
        }
    }
}
