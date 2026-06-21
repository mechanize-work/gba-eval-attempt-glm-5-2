fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}
fn rd32(buf: &[u8], off: usize) -> u32 {
    rd16(buf, off) as u32 | ((rd16(buf, off+2) as u32) << 16)
}

fn main() {
    let rom_data = std::fs::read("dev-roms/another-world.gba").expect("ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    // Run 3 frames to get past BIOS
    for _ in 0..3 {
        gba_emu::emulator::run_frame();
    }
    
    // Check DMA state at frame 3 (where palette gets overwritten)
    let emu = gba_emu::emulator::get_emu();
    let io: &[u8] = &emu.mem.io[..];
    
    eprintln!("DMA state after frame 2:");
    for ch in 0..4 {
        let src = rd32(io, 0xB0 + ch * 0xC);
        let dst = rd32(io, 0xB4 + ch * 0xC);
        let cnt = rd32(io, 0xB8 + ch * 0xC);
        let enabled = cnt & 0x80000000 != 0;
        let start_mode = (cnt >> 28) & 3;
        eprintln!("  DMA{}: src=0x{:08X} dst=0x{:08X} cnt=0x{:08X} en={} start={}",
            ch, src, dst, cnt, enabled, start_mode);
    }
    
    // Now check palette[0] before and after frame 3
    let pal0_before: u16 = emu.mem.palette[0] as u16 | ((emu.mem.palette[1] as u16) << 8);
    eprintln!("Palette[0] before frame 3: 0x{:04X}", pal0_before);
    
    gba_emu::emulator::run_frame();
    
    let emu = gba_emu::emulator::get_emu();
    let pal0_after: u16 = emu.mem.palette[0] as u16 | ((emu.mem.palette[1] as u16) << 8);
    eprintln!("Palette[0] after frame 3: 0x{:04X}", pal0_after);
    
    // Check DMA state after frame 3
    let io: &[u8] = &emu.mem.io[..];
    eprintln!("DMA state after frame 3:");
    for ch in 0..4 {
        let src = rd32(io, 0xB0 + ch * 0xC);
        let dst = rd32(io, 0xB4 + ch * 0xC);
        let cnt = rd32(io, 0xB8 + ch * 0xC);
        let enabled = cnt & 0x80000000 != 0;
        let start_mode = (cnt >> 28) & 3;
        eprintln!("  DMA{}: src=0x{:08X} dst=0x{:08X} cnt=0x{:08X} en={} start={}",
            ch, src, dst, cnt, enabled, start_mode);
    }
}
