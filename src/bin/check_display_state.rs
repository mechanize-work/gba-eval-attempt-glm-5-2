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
    
    // Step until DISPCNT changes
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != 0x0080 && dc != 0x0000 { break; }
    }
    
    // Run 5 frames
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Check display state
    let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    let snap_dc = emu.ppu.snap_dispcnt;
    eprintln!("DISPCNT=0x{:04X} snap=0x{:04X}", dc, snap_dc);
    
    // BG control
    for i in 0..4 {
        let bgcnt = (emu.mem.io[0x08 + i*2] as u16) | ((emu.mem.io[0x09 + i*2] as u16) << 8);
        let hofs = (emu.mem.io[0x10 + i*4] as u16) | ((emu.mem.io[0x11 + i*4] as u16) << 8);
        let vofs = (emu.mem.io[0x12 + i*4] as u16) | ((emu.mem.io[0x13 + i*4] as u16) << 8);
        eprintln!("BG{}: CNT=0x{:04X} HOFS={} VOFS={}", i, bgcnt, hofs & 0x1FF, vofs & 0x1FF);
    }
    
    // Palette
    let pal0 = (emu.mem.palette[0] as u16) | ((emu.mem.palette[1] as u16) << 8);
    let pal1 = (emu.mem.palette[2] as u16) | ((emu.mem.palette[3] as u16) << 8);
    eprintln!("\nPalette: [0]=0x{:04X} [1]=0x{:04X}", pal0, pal1);
    
    // Count non-zero palette entries
    let mut pal_nonzero = 0;
    for i in 0..256 {
        let val = (emu.mem.palette[i*2] as u16) | ((emu.mem.palette[i*2+1] as u16) << 8);
        if val != 0 { pal_nonzero += 1; }
    }
    eprintln!("BG palette non-zero: {}/256", pal_nonzero);
    
    // Count non-zero VRAM
    let mut vram_nonzero = 0;
    for i in 0..VRAM_SIZE {
        if emu.mem.vram[i] != 0 { vram_nonzero += 1; }
    }
    eprintln!("VRAM non-zero: {}/{}", vram_nonzero, VRAM_SIZE);
    
    // Check first few VRAM bytes
    eprintln!("\nVRAM[0..16]:");
    for i in 0..16 {
        eprint!("{:02X} ", emu.mem.vram[i]);
    }
    eprintln!();
}

use gba_emu::memory::VRAM_SIZE;
