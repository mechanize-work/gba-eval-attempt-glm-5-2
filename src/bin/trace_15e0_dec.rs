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
    
    // Step until halted, then step until wake, then trace
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        if emu.cpu.halted {
            // Step until wake
            for j in 0..300000u64 {
                gba_emu::emulator::step_one();
                if !emu.cpu.halted {
                    // Trace 100 instructions after wake
                    let mut last_15e0 = emu.mem.read_word(0x030015E0);
                    let mut last_dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                    let mut last_vb = emu.mem.read_word(0x03003E5C);
                    
                    for k in 0..500u32 {
                        let pc = emu.cpu.r[15];
                        let v15e0 = emu.mem.read_word(0x030015E0);
                        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                        let vb = emu.mem.read_word(0x03003E5C);
                        
                        if v15e0 != last_15e0 {
                            eprintln!("[{}] [15E0] 0x{:08X} -> 0x{:08X} at PC=0x{:08X}", k, last_15e0, v15e0, pc);
                            last_15e0 = v15e0;
                        }
                        if dc != last_dc {
                            eprintln!("[{}] DC 0x{:04X} -> 0x{:04X} at PC=0x{:08X}", k, last_dc, dc, pc);
                            last_dc = dc;
                        }
                        if vb != last_vb {
                            eprintln!("[{}] VB_h 0x{:08X} -> 0x{:08X} at PC=0x{:08X}", k, last_vb, vb, pc);
                            last_vb = vb;
                        }
                        
                        // Show key PCs
                        if pc == 0x08000726 || pc == 0x0800070C || pc == 0x08000714 || pc == 0x08000716 {
                            eprintln!("[{}] PC=0x{:08X} [15E0]=0x{:08X} R3={:08X}", k, pc, v15e0, emu.cpu.r[3]);
                        }
                        
                        gba_emu::emulator::step_one();
                        if emu.cpu.halted { eprintln!("[{}] HALTED", k+1); break; }
                    }
                    return;
                }
            }
            break;
        }
    }
}
