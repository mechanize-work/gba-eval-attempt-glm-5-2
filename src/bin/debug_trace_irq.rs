fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}

fn main() {
    let rom_data = std::fs::read("dev-roms/anguna.gba").expect("Failed to read ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    // Run 3 frames to get to the halted state
    for _ in 0..3 {
        gba_emu::emulator::run_frame();
    }
    
    // Now run frame 4 step by step, tracing IRQ mode instructions
    let emu = gba_emu::emulator::get_emu();
    eprintln!("State before frame 4: PC=0x{:08X} cpsr=0x{:08X} halted={} vbwait={}",
        emu.cpu.r[15], emu.cpu.cpsr, emu.cpu.halted, emu.cpu.vblank_intr_wait);
    
    let target_cycles = emu.cycle_count + 280896;
    let mut count = 0;
    let mut irq_count = 0;
    
    while emu.cycle_count < target_cycles && count < 500000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            if emu.cpu.vblank_intr_wait {
                let cycles_to_vblank = if emu.current_scanline < 160 {
                    (160 - emu.current_scanline as u32) * 1232 - emu.cycle_in_scanline
                } else {
                    (228 - emu.current_scanline as u32 + 160) * 1232 - emu.cycle_in_scanline
                };
                let remaining = target_cycles.wrapping_sub(emu.cycle_count);
                let advance = cycles_to_vblank.min(remaining).max(1);
                emu.cycle_count += advance;
                emu.advance_hardware(advance);
            } else {
                let advance = (1232 - emu.cycle_in_scanline).min(target_cycles.wrapping_sub(emu.cycle_count)).max(1);
                emu.cycle_count += advance;
                emu.advance_hardware(advance);
            }
            count += 1;
        } else {
            let mode = emu.cpu.cpsr & 0x1F;
            let pc = emu.cpu.r[15];
            
            // Trace IRQ mode instructions
            if mode == 0x12 { // MODE_IRQ
                irq_count += 1;
                if irq_count <= 50 {
                    let thumb = emu.cpu.is_thumb();
                    let instr = if thumb {
                        emu.mem.read_half(pc) as u32
                    } else {
                        emu.mem.read_word(pc)
                    };
                    eprintln!("IRQ[{}] PC=0x{:08X} instr=0x{:04X} thumb={} cpsr=0x{:08X} LR=0x{:08X} SP=0x{:08X}",
                        irq_count, pc, instr, thumb, emu.cpu.cpsr, emu.cpu.r[14], emu.cpu.r[13]);
                }
            }
            
            // Also trace right after IRQ mode ends
            if mode != 0x12 && irq_count > 0 && irq_count <= 55 {
                irq_count += 1;
                let thumb = emu.cpu.is_thumb();
                let instr = if thumb {
                    emu.mem.read_half(pc) as u32
                } else {
                    emu.mem.read_word(pc)
                };
                eprintln!("POST-IRQ[{}] PC=0x{:08X} instr=0x{:04X} thumb={} cpsr=0x{:08X} mode=0x{:X}",
                    irq_count, pc, instr, thumb, emu.cpu.cpsr, mode);
            }
            
            emu.execute_one();
            emu.check_and_handle_interrupts();
            count += 1;
        }
    }
    
    emu.cycle_count -= 280896;
    emu.ppu.render_frame(&emu.mem);
    emu.frame_count += 1;
    
    let io: &[u8] = &emu.mem.io[..];
    eprintln!("\nAfter frame 4: PC=0x{:08X} cpsr=0x{:08X} IE=0x{:04X} IME=0x{:04X}",
        emu.cpu.r[15], emu.cpu.cpsr, rd16(io, 0x200), rd16(io, 0x208));
}
