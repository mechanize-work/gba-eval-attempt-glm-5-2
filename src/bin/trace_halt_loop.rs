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
    
    // Run 1 frame
    gba_emu::emulator::run_frame();
    eprintln!("After frame 0: halted={} PC=0x{:08X}", emu.cpu.halted, emu.cpu.r[15]);
    
    // Step until halted (VBlankIntrWait)
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X} vbiw={} vb_occ={} scanline={}",
                i, emu.cpu.r[15], emu.cpu.vblank_intr_wait, emu.vblank_occurred, emu.current_scanline);
            
            // Now step through the halt, tracking VBlank
            let mut last_scan = emu.current_scanline;
            for j in 0..300000u64 {
                gba_emu::emulator::step_one();
                
                if emu.current_scanline != last_scan {
                    if emu.current_scanline == 160 {
                        eprintln!("[+{}] VBlank! scanline=160 vb_occ={} halted={} vbiw={}",
                            j, emu.vblank_occurred, emu.cpu.halted, emu.cpu.vblank_intr_wait);
                    }
                    last_scan = emu.current_scanline;
                }
                
                if !emu.cpu.halted {
                    eprintln!("[+{}] CPU woke up! PC=0x{:08X} [15E0]=0x{:08X}",
                        j, emu.cpu.r[15], emu.mem.read_word(0x030015E0));
                    break;
                }
                
                if j % 50000 == 0 && j > 0 {
                    eprintln!("[+{}] Still halted. scanline={} vb_occ={} vbiw={}",
                        j, emu.current_scanline, emu.vblank_occurred, emu.cpu.vblank_intr_wait);
                }
            }
            break;
        }
    }
}
