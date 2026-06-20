fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}
fn rd32(buf: &[u8], off: usize) -> u32 {
    u16::from_le_bytes([buf[off], buf[off+1]]) as u32 | ((u16::from_le_bytes([buf[off+2], buf[off+3]]) as u32) << 16)
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
    
    for frame in 0..10 {
        gba_emu::emulator::run_frame();
        
        let emu = gba_emu::emulator::get_emu();
        let io: &[u8] = &emu.mem.io[..];
        let iwram: &[u8] = &emu.mem.iwram[..];
        let oam: &[u8] = &emu.mem.oam[..];
        let vram: &[u8] = &emu.mem.vram[..];
        
        let mut oam_nz = 0;
        for b in oam.iter() { if *b != 0 { oam_nz += 1; } }
        
        // Count non-zero VRAM in tile/sprite area (0x10000+)
        let mut vram_sprite_nz = 0;
        for b in 0x10000..0x18000 {
            if vram[b] != 0 { vram_sprite_nz += 1; }
        }
        
        // Count non-zero VRAM in BG tile area (0x0000-0xFFFF)
        let mut vram_bg_nz = 0;
        for b in 0..0x10000 {
            if vram[b] != 0 { vram_bg_nz += 1; }
        }
        
        eprintln!("Frame {}: PC=0x{:08X} halted={} vbwait={} IE=0x{:04X} IF=0x{:04X} IME=0x{:04X} DISPCNT=0x{:04X} OAM_nz={} VRAM_bg_nz={} VRAM_sprite_nz={} IRQ_ptr=0x{:08X}",
            frame, emu.cpu.r[15], emu.cpu.halted, emu.cpu.vblank_intr_wait,
            rd16(io, 0x200), rd16(io, 0x202), rd16(io, 0x208), rd16(io, 0),
            oam_nz, vram_bg_nz, vram_sprite_nz, rd32(iwram, 0x7FFC));
    }
}
