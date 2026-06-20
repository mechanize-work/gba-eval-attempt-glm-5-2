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
    
    for _ in 0..4 {
        gba_emu::emulator::run_frame();
    }
    
    let emu = gba_emu::emulator::get_emu();
    let pal: &[u8] = &emu.mem.palette[..];
    
    eprintln!("BG Palette (non-zero, >= 0xF0):");
    for i in 0..256 {
        let val = rd16(pal, i * 2);
        if val != 0 && i >= 0xF0 {
            let r = (val & 0x1F) as u8;
            let g = ((val >> 5) & 0x1F) as u8;
            let b = ((val >> 10) & 0x1F) as u8;
            let r8 = (r << 3) | (r >> 2);
            let g8 = (g << 3) | (g >> 2);
            let b8 = (b << 3) | (b >> 2);
            eprintln!("  BG Pal[{}]: 0x{:04X} -> ({},{},{})", i, val, r8, g8, b8);
        }
    }
    
    // Check VRAM first bytes
    let vram: &[u8] = &emu.mem.vram[..];
    eprintln!("\nVRAM[0..16]: {:02X?}", &vram[0..16]);
}
