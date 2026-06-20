fn main() {
    let rom_data = std::fs::read("dev-roms/meteorain.gba").expect("ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    for _ in 0..4 {
        gba_emu::emulator::run_frame();
    }
    
    let emu = gba_emu::emulator::get_emu();
    let target = emu.cycle_count + 280896;
    let mut count = 0u64;
    let mut history: Vec<(u32, u16, u32, u32, u32)> = Vec::new();
    let mut loop_skip = false;
    
    while emu.cycle_count < target && count < 2_000_000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            if emu.cpu.vblank_intr_wait {
                let cycles_to_vblank = if emu.current_scanline < 160 {
                    (160 - emu.current_scanline as u32) * 1232 - emu.cycle_in_scanline
                } else {
                    (228 - emu.current_scanline as u32 + 160) * 1232 - emu.cycle_in_scanline
                };
                let remaining = target.wrapping_sub(emu.cycle_count);
                let advance = cycles_to_vblank.min(remaining).max(1);
                emu.cycle_count += advance;
                emu.advance_hardware(advance);
            } else {
                let advance = (1232 - emu.cycle_in_scanline).min(target.wrapping_sub(emu.cycle_count)).max(1);
                emu.cycle_count += advance;
                emu.advance_hardware(advance);
            }
            count += 1;
            continue;
        }
        
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr = if thumb { emu.mem.read_half(pc) } else { emu.mem.read_word(pc) as u16 };
        
        // Detect memset loop at 0x080236B4-0x080236C0
        if pc >= 0x080236B4 && pc <= 0x080236C0 {
            if !loop_skip {
                let r2 = emu.cpu.r[2];
                let r6 = emu.cpu.r[6];
                let bytes_left = r6.wrapping_sub(r2);
                let iters = bytes_left / 16;
                let cycles_est = iters * 18; // ~18 cycles per iteration
                eprintln!("MEMSET LOOP: R2=0x{:08X} R6=0x{:08X} bytes={} iters={} est_cycles={}",
                    r2, r6, bytes_left, iters, cycles_est);
                loop_skip = true;
            }
            // Fast-forward: add cycles and advance hardware
            let r2 = emu.cpu.r[2];
            let r6 = emu.cpu.r[6];
            if r2 >= r6 {
                // Loop done
                loop_skip = false;
            } else {
                // Skip ahead by computing remaining iterations
                let bytes_left = r6.wrapping_sub(r2);
                let iters_left = bytes_left / 16;
                let skip_cycles = (iters_left * 18) as u32;
                let remaining = target.wrapping_sub(emu.cycle_count);
                if skip_cycles >= remaining {
                    // Can't skip all - just advance to end of frame
                    emu.cycle_count = target;
                    break;
                }
                emu.cycle_count += skip_cycles;
                emu.advance_hardware(skip_cycles);
                emu.cpu.r[2] = r6 as u32; // Complete the fill
                emu.cpu.r[15] = 0x080236C2u32; // Skip past loop
                count += (iters_left * 7) as u64;
                continue;
            }
        }
        
        // Track instructions outside the loop
        history.push((pc, instr, emu.cpu.r[13], emu.cpu.r[14], emu.cpu.cpsr));
        if history.len() > 30 { history.remove(0); }
        
        // Check for stack overflow
        if emu.cpu.r[13] < 0x03007000 && emu.cpu.r[13] > 0x03000000 {
            eprintln!("STACK OVERFLOW! SP=0x{:08X} PC=0x{:08X}", emu.cpu.r[13], pc);
            for (i, (hpc, hinstr, hsp, hlr, hcpsr)) in history.iter().enumerate() {
                eprintln!("  [{}] PC=0x{:08X} i=0x{:04X} SP=0x{:08X} LR=0x{:08X} cpsr=0x{:08X}",
                    i, hpc, hinstr, hsp, hlr, hcpsr);
            }
            break;
        }
        
        // Check for invalid PC
        if pc < 0x02000000 && pc >= 0x400 || (pc >= 0x04000000 && pc < 0x08000000) || pc >= 0x0E000000 {
            eprintln!("INVALID PC=0x{:08X} cpsr=0x{:08X}", pc, emu.cpu.cpsr);
            for (i, (hpc, hinstr, hsp, hlr, hcpsr)) in history.iter().enumerate() {
                eprintln!("  [{}] PC=0x{:08X} i=0x{:04X} SP=0x{:08X} LR=0x{:08X} cpsr=0x{:08X}",
                    i, hpc, hinstr, hsp, hlr, hcpsr);
            }
            break;
        }
        
        emu.execute_one();
        emu.check_and_handle_interrupts();
        count += 1;
    }
    eprintln!("Done: {} instrs, PC=0x{:08X} SP=0x{:08X}", count, emu.cpu.r[15], emu.cpu.r[13]);
}
