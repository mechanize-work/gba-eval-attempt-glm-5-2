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
    
    // Run frame 1, tracking when PC enters 0x07xxxxxx
    let target = emu.cycle_count.wrapping_add(280896);
    let mut count = 0u64;
    
    while emu.cycle_count < target && count < 500000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
        } else {
            let pc_before = emu.cpu.r[15];
            emu.execute_one();
            emu.check_and_handle_interrupts();
            
            let pc_after = emu.cpu.r[15];
            
            // Detect jump to OAM region
            if pc_after >= 0x07000000 && pc_after < 0x08000000 && pc_before < 0x07000000 {
                eprintln!("[{}] JUMP to OAM! 0x{:08X} -> 0x{:08X} LR=0x{:08X} SP=0x{:08X}",
                    count, pc_before, pc_after, emu.cpu.r[14], emu.cpu.r[13]);
                
                // Trace backwards: what instruction caused the jump?
                let thumb = emu.cpu.is_thumb();
                let instr_before: u32 = if thumb { emu.mem.read_half(pc_before) as u32 } else { emu.mem.read_word(pc_before) };
                eprintln!("  Instruction at 0x{:08X}: 0x{:X} ({})", pc_before, instr_before, if thumb {"T"} else {"A"});
                
                // Check what's at the OAM address
                eprintln!("  OAM[0x{:08X}] = 0x{:08X}", pc_after, emu.mem.read_word(pc_after & !3));
                
                // Trace 20 instructions from OAM
                for j in 0..20u32 {
                    let pc = emu.cpu.r[15];
                    let t = emu.cpu.is_thumb();
                    let instr: u32 = if t { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
                    eprintln!("  [{}] PC=0x{:08X} 0x{:X} {} R0={:08X} LR={:08X}",
                        j, pc, instr, if t {"T"} else {"A"}, emu.cpu.r[0], emu.cpu.r[14]);
                    emu.execute_one();
                }
                break;
            }
        }
        count += 1;
    }
}
