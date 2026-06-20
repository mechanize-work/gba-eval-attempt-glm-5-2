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
        let dispcnt = rd16(io, 0);
        let bg2cnt = rd16(io, 0xC);
        
        // BG2CNT: bits 2-3 = char base, bits 8-12 = screen base
        let char_base = ((bg2cnt >> 2) & 0x3) as usize * 0x4000;
        let screen_base = ((bg2cnt >> 8) & 0x1F) as usize * 0x800;
        let bg_mode = dispcnt & 0x7;
        
        eprintln!("Frame {}: DISPCNT={:04X} BG2CNT={:04X} mode={} char_base=0x{:04X} screen_base=0x{:04X}",
            frame, dispcnt, bg2cnt, bg_mode, char_base, screen_base);
        
        // Check screen base for non-zero entries
        let vram: &[u8] = &emu.mem.vram[..];
        let mut nonzero_tiles = 0;
        for t in 0..1024 {  // 32x32 tiles, each 2 bytes
            let off = screen_base + t * 2;
            if off + 1 < vram.len() {
                let val = rd16(vram, off);
                if val != 0 {
                    nonzero_tiles += 1;
                    if nonzero_tiles <= 3 {
                        eprintln!("  Tile entry {}: {:04X} (tile={}, pal={}, flip={}{}{})",
                            t, val, val & 0x3FF, (val >> 12) & 0xF, 
                            if val & 0x400 != 0 {"H"} else {""},
                            if val & 0x800 != 0 {"V"} else {""},
                            if val & 0x1000 != 0 {"??"} else {""});
                    }
                }
            }
        }
        eprintln!("  Non-zero tile entries: {}", nonzero_tiles);
        
        // Check char base for non-zero tile data
        let mut nonzero_chars = 0;
        for b in 0..32 {
            if vram[char_base + b] != 0 {
                nonzero_chars += 1;
            }
        }
        eprintln!("  First 32 bytes of char data at 0x{:04X}: {} non-zero", char_base, nonzero_chars);
        
        // Check BG palette
        let pal: &[u8] = &emu.mem.palette[..];
        let mut nonzero_pals = 0;
        for p in 0..256 {
            let val = rd16(pal, p * 2);
            if val != 0 {
                nonzero_pals += 1;
                if nonzero_pals <= 3 {
                    eprintln!("  BG pal[{}]: {:04X}", p, val);
                }
            }
        }
        eprintln!("  Non-zero BG palette entries: {}", nonzero_pals);
        
        // Check BG scroll registers
        let bg2hofs = rd16(io, 0x18);
        let bg2vofs = rd16(io, 0x1A);
        eprintln!("  BG2HOFS={} BG2VOFS={}", bg2hofs & 0x1FF, bg2vofs & 0x1FF);
    }
}
