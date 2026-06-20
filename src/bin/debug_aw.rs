fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}

fn main() {
    let rom_data = std::fs::read("dev-roms/another-world.gba").expect("Failed to read ROM");
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
        let dispcnt = rd16(io, 0);
        let bg0cnt = rd16(io, 8);
        let bg2cnt = rd16(io, 0xC);
        
        // Check first few VRAM halfwords (for mode 3/5 bitmap)
        let vram: &[u8] = &emu.mem.vram[..];
        let v0 = rd16(vram, 0);
        let v1 = rd16(vram, 2);
        
        eprintln!("Frame {}: DISPCNT=0x{:04X} BG0CNT=0x{:04X} BG2CNT=0x{:04X} VRAM[0]=0x{:04X} VRAM[2]=0x{:04X}",
            frame, dispcnt, bg0cnt, bg2cnt, v0, v1);
    }
}
