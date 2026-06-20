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
    
    // Run frames until VBlankIntrWait is reached
    for frame in 0..10 {
        gba_emu::emulator::run_frame();
        eprintln!("Frame {}: PC=0x{:08X} halted={} scanline={} vb_occ={} dc=0x{:04X}",
            frame, emu.cpu.r[15], emu.cpu.halted, emu.current_scanline, emu.vblank_occurred,
            (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
    }
    
    // Now trace the halted state
    if emu.cpu.halted {
        eprintln!("\nCPU is halted. Tracing VBlank wake-up...");
        let mut last_scan = emu.current_scanline;
        for j in 0..300000u64 {
            gba_emu::emulator::step_one();
            
            if emu.current_scanline != last_scan {
                if emu.current_scanline == 160 {
                    eprintln!("[+{}] VBlank! scanline=160 vb_occ={} halted={}",
                        j, emu.vblank_occurred, emu.cpu.halted);
                }
                last_scan = emu.current_scanline;
            }
            
            if !emu.cpu.halted {
                eprintln!("[+{}] CPU woke up! PC=0x{:08X} scanline={}",
                    j, emu.cpu.r[15], emu.current_scanline);
                break;
            }
            
            if j % 100000 == 0 && j > 0 {
                eprintln!("[+{}] Still halted. scanline={} vb_occ={}",
                    j, emu.current_scanline, emu.vblank_occurred);
            }
        }
    }
}
