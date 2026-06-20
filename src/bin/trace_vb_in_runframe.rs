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
    
    // Run 2 frames using run_frame
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    eprintln!("After 2 frames: halted={} vbiw={} vb_occ={} scanline={} cc={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred, emu.current_scanline, emu.cycle_count);
    
    // Now use run_frame for frame 2 but add debug inside
    // We can't modify run_frame, so let's check what state it produces
    gba_emu::emulator::run_frame();
    
    eprintln!("After frame 2: halted={} vbiw={} vb_occ={} scanline={} cc={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred, emu.current_scanline, emu.cycle_count,
        emu.mem.read_word(0x030015E0));
    
    // The issue: run_frame uses self.cycle_count which wraps.
    // Let me check if cycle_count wrapping is causing the while loop to exit early.
    // After frame 1: cc = 2 (after wrapping_sub)
    // Target for frame 2: 2 + 280896 = 280898
    // But what if cc overflows during the frame?
    // 280898 is well within u32 range, so no overflow.
    
    // Let me check if the VBlankIntrWait check is even reached
    // by checking if vblank_occurred is ever true during run_frame
    
    // Actually, the issue might be that advance_hardware sets vb_occ=true
    // but then check_and_handle_interrupts checks it and clears it,
    // but the VBlankIntrWait path requires halted && vbiw && vb_occ.
    // If the regular IRQ path fires first (because IME=1, IE=0x0001, IF has VBlank),
    // the VBlankIntrWait path never runs.
    
    // In advance_hardware, when scanline hits 160:
    // 1. self.irq.signal(IRQ_VBLANK) -> self.irq.if_ |= 1
    // 2. self.vblank_occurred = true
    // 3. self.sync_interrupts() -> writes IF to IO memory
    //
    // Then in check_and_handle_interrupts:
    // 1. Sync IF from IO (now has VBlank bit)
    // 2. Check pending(): IE & IF = 0x0001 (true!)
    // 3. raise_irq() -> CPU enters IRQ mode
    // 4. The VBlankIntrWait check (halted && vbiw && vb_occ) is NEVER reached
    //    because the regular pending() check fires first!
    
    eprintln!("\nIE=0x{:04X} IF=0x{:04X} IME={}",
        (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8),
        (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8),
        (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8));
    eprintln!("CPSR=0x{:08X} I_flag={}", emu.cpu.cpsr, emu.cpu.cpsr & 0x80 != 0);
}
