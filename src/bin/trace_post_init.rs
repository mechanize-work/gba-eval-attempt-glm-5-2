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
    
    // Step past the EWRAM clear (first memset with R1=0x40000)
    // and the BSS clear (second memset with R1=0x29BC)
    // Count memset exits
    let mut in_memset = false;
    let mut memset_exits = 0u32;
    
    for i in 0..50_000_000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        
        let was_in = in_memset;
        in_memset = pc >= 0x08000186 && pc <= 0x08000196;
        
        if was_in && !in_memset {
            memset_exits += 1;
            eprintln!("Memset exit #{} at step {}: PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                memset_exits, i, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
            
            if memset_exits >= 4 {
                // After 4th exit, trace 200 instructions
                eprintln!("\nTracing after 4th memset exit:");
                for j in 0..200u32 {
                    let pc2 = emu.cpu.r[15];
                    let thumb = emu.cpu.is_thumb();
                    let instr: u32 = if thumb { emu.mem.read_half(pc2) as u32 } else { emu.mem.read_word(pc2) };
                    let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                    let halted = emu.cpu.halted;
                    let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
                    let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
                    
                    // Check for SWI
                    let swi = if thumb && (instr & 0xFF00) == 0xDF00 {
                        format!(" *** SWI {} ***", instr & 0xFF)
                    } else { String::new() };
                    
                    eprintln!("[{:3}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X} DC=0x{:04X} h={} IME={} IE=0x{:04X}{}",
                        j, pc2, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                        emu.cpu.r[13], emu.cpu.r[14], dispcnt, halted, ime, ie, swi);
                    
                    gba_emu::emulator::step_one();
                    
                    if emu.cpu.halted {
                        eprintln!("[{}] HALTED", j+1);
                        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
                        eprintln!("  IF=0x{:04X}", if_);
                        break;
                    }
                    
                    let pc3 = emu.cpu.r[15];
                    if pc3 >= 0x08000186 && pc3 <= 0x08000196 {
                        eprintln!("[{}] Back in memset. R1={:08X}", j+1, emu.cpu.r[1]);
                        break;
                    }
                }
                break;
            }
            in_memset = true;
        }
        
        if i % 10_000_000 == 0 && i > 0 {
            eprintln!("Step {}: PC=0x{:08X} R1={:08X}", i, pc, emu.cpu.r[1]);
        }
    }
}
