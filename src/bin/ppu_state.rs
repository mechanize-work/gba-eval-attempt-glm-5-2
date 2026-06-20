// Dump PPU state after N frames
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

    // Run 5 frames
    for i in 0..5 {
        gba_emu::emulator::run_frame();
    }

    let emu = gba_emu::emulator::get_emu();
    
    let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    let bg0cnt = (emu.mem.io[0x08] as u16) | ((emu.mem.io[0x09] as u16) << 8);
    let bg1cnt = (emu.mem.io[0x0A] as u16) | ((emu.mem.io[0x0B] as u16) << 8);
    let bg2cnt = (emu.mem.io[0x0C] as u16) | ((emu.mem.io[0x0D] as u16) << 8);
    let bg3cnt = (emu.mem.io[0x0E] as u16) | ((emu.mem.io[0x0F] as u16) << 8);
    
    eprintln!("After 5 frames:");
    eprintln!("  DISPCNT=0x{:04X} mode={} bg_en=0x{:X} obj_en={} obj_1d={}",
        dispcnt, dispcnt & 7, (dispcnt >> 8) & 0xF, (dispcnt >> 12) & 1, (dispcnt >> 6) & 1);
    eprintln!("  BG0CNT=0x{:04X} priority={} char_base={} screen_base={} palette_256={} size={}",
        bg0cnt, bg0cnt & 3, (bg0cnt >> 2) & 3, (bg0cnt >> 8) & 0x1F, (bg0cnt >> 7) & 1, (bg0cnt >> 14) & 3);
    eprintln!("  BG1CNT=0x{:04X}", bg1cnt);
    eprintln!("  BG2CNT=0x{:04X}", bg2cnt);
    eprintln!("  BG3CNT=0x{:04X}", bg3cnt);
    
    let bg0hofs = (emu.mem.io[0x10] as u16) | ((emu.mem.io[0x11] as u16) << 8);
    let bg0vofs = (emu.mem.io[0x12] as u16) | ((emu.mem.io[0x13] as u16) << 8);
    eprintln!("  BG0HOFS={} BG0VOFS={}", bg0hofs, bg0vofs);
    
    let bldcnt = (emu.mem.io[0x50] as u16) | ((emu.mem.io[0x51] as u16) << 8);
    eprintln!("  BLDCNT=0x{:04X}", bldcnt);
    
    // Check palette
    eprintln!("  BG palette[0]=0x{:04X}", (emu.mem.palette[0] as u16) | ((emu.mem.palette[1] as u16) << 8));
    eprintln!("  OBJ palette[0]=0x{:04X}", (emu.mem.palette[0x200] as u16) | ((emu.mem.palette[0x201] as u16) << 8));
    
    // Check some palette entries
    for i in 0..10 {
        let off = i * 2;
        let bg_color = (emu.mem.palette[off] as u16) | ((emu.mem.palette[off+1] as u16) << 8);
        let obj_color = (emu.mem.palette[0x200 + off] as u16) | ((emu.mem.palette[0x201 + off] as u16) << 8);
        eprintln!("  BG pal[{}]=0x{:04X}  OBJ pal[{}]=0x{:04X}", i, bg_color, i, obj_color);
    }
    
    // Check OAM - first few sprites
    eprintln!("\n  OAM entries:");
    for i in 0..10 {
        let off = i * 8;
        let attr0 = (emu.mem.oam[off] as u16) | ((emu.mem.oam[off+1] as u16) << 8);
        let attr1 = (emu.mem.oam[off+2] as u16) | ((emu.mem.oam[off+3] as u16) << 8);
        let attr2 = (emu.mem.oam[off+4] as u16) | ((emu.mem.oam[off+5] as u16) << 8);
        if attr0 != 0 || attr1 != 0 || attr2 != 0 {
            let y = attr0 & 0xFF;
            let shape = (attr0 >> 14) & 3;
            let x = attr1 & 0x1FF;
            let size = (attr1 >> 14) & 3;
            let tile = attr2 & 0x3FF;
            let prio = (attr2 >> 10) & 3;
            let pal = (attr2 >> 12) & 0xF;
            let mode = (attr0 >> 10) & 3;
            let bpp8 = (attr0 >> 13) & 1;
            eprintln!("    [{}] y={} x={} shape={} size={} tile={} prio={} pal={} mode={} 8bpp={} (attr0=0x{:04X} attr1=0x{:04X} attr2=0x{:04X})",
                i, y, x, shape, size, tile, prio, pal, mode, bpp8, attr0, attr1, attr2);
        }
    }
    
    // Check VRAM - first tiles at char base
    let char_base = ((bg0cnt >> 2) & 3) as u32 * 0x4000;
    eprintln!("\n  VRAM char_base=0x{:05X}", char_base);
    for i in 0..8 {
        let addr = char_base + i;
        eprintln!("    VRAM[0x{:05X}]=0x{:02X}", addr, emu.mem.vram[addr as usize]);
    }
    
    // Check what's at framebuffer
    let fb = &emu.ppu.framebuffer;
    let non_black = fb.iter().filter(|p| **p != 0 && **p != 0xFF000000).count();
    eprintln!("\n  Framebuffer: {} non-black pixels", non_black);
    if non_black > 0 {
        for i in 0..38400 {
            if fb[i] != 0 && fb[i] != 0xFF000000 {
                let x = i % 240;
                let y = i / 240;
                eprintln!("    First non-black pixel at ({},{}): 0x{:08X}", x, y, fb[i]);
                break;
            }
        }
    }
}
