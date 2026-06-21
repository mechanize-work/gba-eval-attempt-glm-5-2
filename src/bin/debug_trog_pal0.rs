fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}

fn main() {
    let rom_data = std::fs::read("dev-roms/trogdor.gba").expect("ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    for _ in 0..8 {
        gba_emu::emulator::run_frame();
    }
    
    let emu = gba_emu::emulator::get_emu();
    let pal: &[u8] = &emu.mem.palette[..];
    eprintln!("Palette[0] = 0x{:04X}", rd16(pal, 0));
    eprintln!("DISPCNT=0x{:04X} WINOUT=0x{:04X}", emu.ppu.snap_dispcnt, emu.ppu.snap_winout);
    
    let vram: &[u8] = &emu.mem.vram[..];
    for y in 93..100 {
        let off = y * 480 + 239 * 2;
        let v = rd16(vram, off);
        eprintln!("  VRAM[(239,{})] = 0x{:04X} (offset=0x{:X})", y, v, off);
    }
}
