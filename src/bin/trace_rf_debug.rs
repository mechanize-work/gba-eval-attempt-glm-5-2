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
    eprintln!("After frame 0: halted={} cc={} scan={}", emu.cpu.halted, emu.cycle_count, emu.current_scanline);
    
    // Run frame 1 - should reach VBlankIntrWait
    gba_emu::emulator::run_frame();
    eprintln!("After frame 1: halted={} vbiw={} cc={} scan={} [15E0]=0x{:08X} IF=0x{:04X} IE=0x{:04X} IME={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cycle_count, emu.current_scanline,
        emu.mem.read_word(0x030015E0),
        (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8),
        (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8),
        (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8));
    
    // Now check what run_frame does with the VBlank IF
    // The issue: run_frame calls self.irq.signal(IRQ_VBLANK) at the end
    // This sets IF bit 0 in the irq struct but NOT in IO memory
    // Then on the next frame, check_and_handle_interrupts reads IF from IO
    // which doesn't have the VBlank bit, so it thinks no VBlank occurred
    // But advance_hardware sets IF at scanline 160, which should work
    
    // Actually, the issue is that advance_hardware signals IRQ_VBLANK
    // which calls self.irq.signal(IRQ_VBLANK) -> self.irq.if_ |= 1
    // But then sync_interrupts writes self.irq.if_ to IO memory
    // So IF should have the VBlank bit.
    
    // Let me check if the VBlankIntrWait check is even reached
    // The check is: halted && vbiw && vblank_occurred
    // Maybe vblank_occurred is never set to true in run_frame
    
    // Run frame 2 with manual tracking
    let start_cc = emu.cycle_count;
    let target = start_cc.wrapping_add(280896);
    let mut count = 0u64;
    let mut last_scan = emu.current_scanline;
    let mut vb_count = 0;
    
    while emu.cycle_count < target && count < 500000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
        } else {
            emu.execute_one();
            emu.check_and_handle_interrupts();
        }
        count += 1;
        
        if emu.current_scanline != last_scan {
            if emu.current_scanline == 160 {
                vb_count += 1;
                eprintln!("[{}] VBlank #{}! vb_occ={} halted={} vbiw={}",
                    count, vb_count, emu.vblank_occurred, emu.cpu.halted, emu.cpu.vblank_intr_wait);
            }
            last_scan = emu.current_scanline;
        }
        
        if !emu.cpu.halted && emu.cpu.r[15] != 0x08008B20 && count > 190000 {
            eprintln!("[{}] CPU running! PC=0x{:08X} [15E0]=0x{:08X}",
                count, emu.cpu.r[15], emu.mem.read_word(0x030015E0));
        }
    }
    
    eprintln!("After manual frame 2: halted={} PC=0x{:08X} [15E0]=0x{:08X} count={} vb_count={}",
        emu.cpu.halted, emu.cpu.r[15], emu.mem.read_word(0x030015E0), count, vb_count);
}
