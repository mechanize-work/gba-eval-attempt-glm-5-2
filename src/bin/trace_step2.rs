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
    
    // Step through and detect loop exits
    let mut last_exit_step = 0u64;
    let mut in_memset = true;
    let mut exit_count = 0u32;
    
    for i in 0..50_000_000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        
        let was_in = in_memset;
        in_memset = pc >= 0x08000186 && pc <= 0x08000196;
        
        if was_in && !in_memset {
            exit_count += 1;
            let steps_since_last = i - last_exit_step;
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            eprintln!("Exit #{} at step {} ({} steps since last): PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} DISPCNT=0x{:04X}",
                exit_count, i, steps_since_last, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[4], dispcnt);
            
            // Trace where it goes
            for j in 0..30 {
                let pc2 = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc2) as u32;
                gba_emu::emulator::step_one();
                
                let pc3 = emu.cpu.r[15];
                if pc3 >= 0x08000186 && pc3 <= 0x08000196 {
                    eprintln!("  Back in memset after {} steps. R0={:08X} R1={:08X}", j+1, emu.cpu.r[0], emu.cpu.r[1]);
                    break;
                }
                
                // Check for SWI
                if (instr & 0xFF00) == 0xDF00 {
                    eprintln!("  SWI {} at step {}", instr & 0xFF, i+j+1);
                }
                
                // Check for halt
                if emu.cpu.halted {
                    eprintln!("  HALTED at step {}", i+j+1);
                    break;
                }
            }
            
            last_exit_step = i;
            in_memset = true;
            
            if exit_count >= 5 {
                break;
            }
        }
        
        if i % 10_000_000 == 0 && i > 0 {
            eprintln!("Step {}: still in memset, R1={:08X}", i, emu.cpu.r[1]);
        }
    }
}
