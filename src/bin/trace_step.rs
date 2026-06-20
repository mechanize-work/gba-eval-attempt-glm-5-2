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
    
    // Step through and detect when PC first leaves the memset loop
    let mut in_memset = true;
    let mut memset_count = 0;
    
    for i in 0..50_000_000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        
        let was_in = in_memset;
        in_memset = pc >= 0x08000190 && pc <= 0x08000196;
        
        if was_in && !in_memset {
            memset_count += 1;
            eprintln!("[{}] Exited memset #{}! PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}",
                i, memset_count, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[13], emu.cpu.r[14]);
            
            // Trace 100 instructions
            for j in 0..100 {
                let pc2 = emu.cpu.r[15];
                let thumb = emu.cpu.is_thumb();
                let instr: u32 = if thumb { emu.mem.read_half(pc2) as u32 } else { emu.mem.read_word(pc2) };
                
                // Check for SWI
                if thumb && (instr & 0xFF00) == 0xDF00 {
                    eprintln!("  [{}+{}] SWI {} at PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                        i, j, instr & 0xFF, pc2, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
                } else if !thumb && (instr >> 24) == 0xEF {
                    eprintln!("  [{}+{}] ARM SWI 0x{:06X} at PC=0x{:08X}",
                        i, j, instr & 0xFFFFFF, pc2);
                }
                
                gba_emu::emulator::step_one();
                
                let pc3 = emu.cpu.r[15];
                if emu.cpu.halted {
                    eprintln!("  [{}+{}] CPU halted at PC=0x{:08X}", i, j+1, pc3);
                    let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
                    let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
                    let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
                    eprintln!("    IME={} IE=0x{:04X} IF=0x{:04X}", ime, ie, if_);
                    break;
                }
                
                // Check if back in memset
                if pc3 >= 0x08000190 && pc3 <= 0x08000196 {
                    eprintln!("  [{}+{}] Back in memset at PC=0x{:08X} R1={:08X}", i, j+1, pc3, emu.cpu.r[1]);
                    break;
                }
            }
            
            if memset_count >= 3 {
                break;
            }
            in_memset = true; // Reset for next detection
        }
        
        if i % 10_000_000 == 0 && i > 0 {
            eprintln!("[{}] Still running... PC=0x{:08X} R1={:08X}", i, pc, emu.cpu.r[1]);
        }
    }
}
