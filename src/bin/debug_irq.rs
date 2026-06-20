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
    
    for frame in 0..5 {
        gba_emu::emulator::run_frame();
        let emu = gba_emu::emulator::get_emu();
        let io: &[u8] = &emu.mem.io[..];
        let ie = rd16(io, 0x200);
        let iflag = rd16(io, 0x202);
        let ime = rd16(io, 0x208);
        eprintln!("Frame {}: IE={:04X} IF={:04X} IME={:04X}", frame, ie, iflag, ime);
        eprintln!("  CPU: halted={} vblank_wait={} pc={:08X} cpsr={:08X}", 
            emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cpu.r[15], emu.cpu.cpsr);
        eprintln!("  mode={:X} irq_disabled={}", emu.cpu.cpsr & 0x1F, (emu.cpu.cpsr >> 7) & 1);
        
        // Check IWRAM interrupt handler pointer
        let iwram: &[u8] = &emu.mem.iwram[..];
        let irq_handler = u32::from_le_bytes([iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]]);
        eprintln!("  IRQ handler ptr at 0x03007FFC: {:08X}", irq_handler);
        
        // Check what's at the interrupt vector in IWRAM (0x03000000)
        let vec0 = u32::from_le_bytes([iwram[0], iwram[1], iwram[2], iwram[3]]);
        let vec1 = u32::from_le_bytes([iwram[4], iwram[5], iwram[6], iwram[7]]);
        eprintln!("  IWRAM[0]: {:08X} IWRAM[4]: {:08X}", vec0, vec1);
        
        // Check OAM for any non-zero
        let oam: &[u8] = &emu.mem.oam[..];
        let mut oam_nonzero = 0;
        for b in oam.iter() {
            if *b != 0 { oam_nonzero += 1; }
        }
        eprintln!("  OAM non-zero bytes: {}/1024", oam_nonzero);
        
        // Check VRAM at various screen bases
        let vram: &[u8] = &emu.mem.vram[..];
        for sb in 0..16 {
            let base = sb * 0x800;
            let mut nz = 0;
            for b in 0..0x800 {
                if vram[base + b] != 0 { nz += 1; }
            }
            if nz > 0 {
                eprintln!("  VRAM screen base 0x{:04X}: {} non-zero bytes", base, nz);
            }
        }
    }
}
