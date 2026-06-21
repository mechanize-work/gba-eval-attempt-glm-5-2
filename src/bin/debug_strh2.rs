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
        
        // Check both STRH R5, [R2, #0] locations
        if emu.cpu.is_thumb() && (pc == 0x08003D08 || pc == 0x08003DE4) {
            // Check if writing to palette[0]
            if emu.cpu.r[2] == 0x05000000 {
                eprintln!("Pal[0] write at PC=0x{:08X}: R5=0x{:08X} R1=0x{:08X} R3=0x{:08X} R2=0x{:08X} count={}",
                    pc, emu.cpu.r[5], emu.cpu.r[1], emu.cpu.r[3], emu.cpu.r[2], count);
            }
        }
        
        emu.execute_one();
        emu.check_and_handle_interrupts();
        count += 1;
    }
}
