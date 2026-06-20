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
    
    // Run 2 frames to get past init
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    // Now step until VBlankIntrWait, then trace what happens after
    let mut found_vbiw = false;
    let mut after_vbiw = false;
    
    for i in 0..500000u64 {
        let pc = emu.cpu.r[15];
        let instr = if emu.cpu.is_thumb() { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let is_vbiw = (instr & 0xFF00) == 0xDF05;
        
        if is_vbiw && !found_vbiw {
            eprintln!("[{}] VBlankIntrWait at PC=0x{:08X}", i, pc);
            found_vbiw = true;
        }
        
        gba_emu::emulator::step_one();
        
        if found_vbiw && !emu.cpu.halted && !after_vbiw {
            after_vbiw = true;
            eprintln!("[{}] After VBlankIntrWait, PC=0x{:08X}", i+1, emu.cpu.r[15]);
            
            // Trace 200 instructions looking for writes to 0x030015E0
            let mut last_15e0 = emu.mem.read_word(0x030015E0);
            for j in 0..500u32 {
                let pc2 = emu.cpu.r[15];
                let instr2 = if emu.cpu.is_thumb() { emu.mem.read_half(pc2) as u32 } else { emu.mem.read_word(pc2) };
                let v15e0 = emu.mem.read_word(0x030015E0);
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                
                if v15e0 != last_15e0 {
                    eprintln!("  [{}] *** [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X} ***", j, last_15e0, v15e0, pc2);
                    last_15e0 = v15e0;
                }
                
                if dc != 0x0080 {
                    eprintln!("  [{}] *** DISPCNT = 0x{:04X} at PC=0x{:08X} ***", j, dc, pc2);
                }
                
                // Show all instructions (there should be few between VBIW calls)
                if j < 50 || (instr2 & 0xFF00) == 0xDF00 || dc != 0x0080 {
                    let is_swi = (instr2 & 0xFF00) == 0xDF00;
                    let extra = if is_swi { format!(" SWI={}", instr2 & 0xFF) } else { String::new() };
                    eprintln!("  [{:3}] PC=0x{:08X} 0x{:X} DC=0x{:04X} R0={:08X} R1={:08X}{}",
                        j, pc2, instr2, dc, emu.cpu.r[0], emu.cpu.r[1], extra);
                }
                
                gba_emu::emulator::step_one();
                if emu.cpu.halted { eprintln!("  [{}] HALTED", j+1); break; }
            }
            break;
        }
    }
}
