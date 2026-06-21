fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}

fn main() {
    let rom_data = std::fs::read("dev-roms/meteorain.gba").expect("ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    // Run 4 frames normally
    for _ in 0..4 {
        gba_emu::emulator::run_frame();
    }
    
    // Now trace frame 5
    let emu = gba_emu::emulator::get_emu();
    eprintln!("Before frame 5: PC=0x{:08X} SP=0x{:08X} halted={} cpsr=0x{:08X}",
        emu.cpu.r[15], emu.cpu.r[13], emu.cpu.halted, emu.cpu.cpsr);
    
    let target = emu.cycle_count + 280896;
    let mut count = 0u64;
    let mut last_pc = 0u32;
    let mut same_pc_count = 0u32;
    
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
        } else {
            let pc = emu.cpu.r[15];
            if pc == last_pc {
                same_pc_count += 1;
                if same_pc_count > 100 {
                    eprintln!("STUCK at PC=0x{:08X} for {} iterations! cpsr=0x{:08X} r0=0x{:08X} r1=0x{:08X} r2=0x{:08X} r3=0x{:08X} lr=0x{:08X} sp=0x{:08X}",
                        pc, same_pc_count, emu.cpu.cpsr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[14], emu.cpu.r[13]);
                    break;
                }
            } else {
                same_pc_count = 0;
                last_pc = pc;
            }
            
            // Print every 100000 instructions
            if count % 500000 == 0 && count > 0 {
                eprintln!("Progress: {} instructions, PC=0x{:08X} SP=0x{:08X} cycles={}",
                    count, pc, emu.cpu.r[13], emu.cycle_count);
            }
            
            emu.execute_one();
            emu.check_and_handle_interrupts();
        }
        count += 1;
    }
    
    eprintln!("Frame 5 done: {} instructions, PC=0x{:08X}", count, emu.cpu.r[15]);
}
