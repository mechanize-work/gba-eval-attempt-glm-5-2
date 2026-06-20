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
    
    // Step until VBlankIntrWait is called
    let mut found_swi = false;
    for i in 0..300000u64 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        
        if (instr & 0xFF00) == 0xDF05 && !found_swi {
            eprintln!("[{}] VBlankIntrWait at PC=0x{:08X}", i, pc);
            eprintln!("  scanline={} cycle_in_scanline={} cycle_count={}",
                emu.current_scanline, emu.cycle_in_scanline, emu.cycle_count);
            found_swi = true;
        }
        
        gba_emu::emulator::step_one();
        
        if found_swi && emu.cpu.halted {
            eprintln!("[{}] After SWI: halted={} scanline={} vb_occ={}",
                i+1, emu.cpu.halted, emu.current_scanline, emu.vblank_occurred);
            
            // Now step through the halt, tracking scanline
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
                
                if j % 50000 == 0 && j > 0 {
                    eprintln!("[+{}] Still halted. scanline={} vb_occ={}",
                        j, emu.current_scanline, emu.vblank_occurred);
                }
            }
            break;
        }
    }
}
