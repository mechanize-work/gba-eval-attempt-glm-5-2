
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
    
    for frame in 0..6 {
        let emu = gba_emu::emulator::get_emu();
        eprintln!("Before frame {}: PC=0x{:08X} SP=0x{:08X} halted={} cpsr=0x{:08X}",
            frame, emu.cpu.r[15], emu.cpu.r[13], emu.cpu.halted, emu.cpu.cpsr);
        
        // Run with a timeout-like check
        let emu = gba_emu::emulator::get_emu();
        let target = emu.cycle_count + 280896;
        let mut count = 0u64;
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
            } else {
                emu.execute_one();
                emu.check_and_handle_interrupts();
                count += 1;
            }
        }
        emu.cycle_count -= 280896;
        emu.ppu.render_frame(&emu.mem);
        emu.frame_count += 1;
        
        let io: &[u8] = &emu.mem.io[..];
        eprintln!("After frame {}: PC=0x{:08X} SP=0x{:08X} halted={} instrs={} IE=0x{:04X} IME=0x{:04X}",
            frame, emu.cpu.r[15], emu.cpu.r[13], emu.cpu.halted, count,
            rd16(io, 0x200), rd16(io, 0x208));
    }
}
