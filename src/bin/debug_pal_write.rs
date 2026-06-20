fn main() {
    let rom_data = std::fs::read("dev-roms/another-world.gba").expect("ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    let emu = gba_emu::emulator::get_emu();
    let target = emu.cycle_count + 280896 * 4;
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
            continue;
        }
        
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let pal_before: u16 = emu.mem.palette[0] as u16 | ((emu.mem.palette[1] as u16) << 8);
        
        emu.execute_one();
        emu.check_and_handle_interrupts();
        
        let pal_after: u16 = emu.mem.palette[0] as u16 | ((emu.mem.palette[1] as u16) << 8);
        if pal_before != pal_after {
            eprintln!("Pal[0]: 0x{:04X} -> 0x{:04X} at PC=0x{:08X} thumb={} count={}",
                pal_before, pal_after, pc, thumb, count);
        }
        
        count += 1;
    }
}
