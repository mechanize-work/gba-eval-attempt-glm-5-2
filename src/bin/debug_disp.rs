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
        let dispstat = rd16(io, 4);
        let vcount = rd16(io, 6);
        let bg0cnt = rd16(io, 8);
        let bg1cnt = rd16(io, 0xA);
        let bg2cnt = rd16(io, 0xC);
        let bg3cnt = rd16(io, 0xE);
        eprintln!("Frame {}: DISPCNT={:04X} DISPSTAT={:04X} VCOUNT={:04X}", 
            frame, dispcnt, dispstat, vcount);
        eprintln!("  BG0CNT={:04X} BG1CNT={:04X} BG2CNT={:04X} BG3CNT={:04X}", 
            bg0cnt, bg1cnt, bg2cnt, bg3cnt);
        
        let bldcnt = rd16(io, 0x50);
        let bldalpha = rd16(io, 0x52);
        let bldy = rd16(io, 0x54);
        let mosaic = rd16(io, 0x4C);
        let winin = rd16(io, 0x48);
        let winout = rd16(io, 0x4A);
        eprintln!("  WININ={:04X} WINOUT={:04X} MOSAIC={:04X}", winin, winout, mosaic);
        eprintln!("  BLDCNT={:04X} BLDALPHA={:04X} BLDY={:04X}", bldcnt, bldalpha, bldy);
        
        // Check OAM (1024 bytes, 128 objects, 8 bytes each)
        let oam: &[u8] = &emu.mem.oam[..];
        let mut obj_count = 0;
        for i in 0..128 {
            let attr0 = rd16(oam, i*8);
            let attr1 = rd16(oam, i*8+2);
            let attr2 = rd16(oam, i*8+4);
            if attr0 & 0x3 != 0x2 {
                obj_count += 1;
                if obj_count <= 5 {
                    let y = attr0 & 0xFF;
                    let x = attr1 & 0x1FF;
                    let shape = (attr0 >> 14) & 0x3;
                    let size = (attr1 >> 14) & 0x3;
                    let tile = attr2 & 0x3FF;
                    let pal = (attr2 >> 12) & 0xF;
                    let obj_mode = (attr0 >> 10) & 0x3;
                    let depth = (attr0 >> 8) & 0x1;
                    eprintln!("  Obj {}: y={} x={} shape={} size={} tile={} pal={} mode={} depth={} a0={:04X} a1={:04X} a2={:04X}",
                        i, y, x, shape, size, tile, pal, obj_mode, depth, attr0, attr1, attr2);
                }
            }
        }
        eprintln!("  Active objects: {}", obj_count);
        
        // Check sprite palette
        let pal: &[u8] = &emu.mem.palette[..];
        for p in 0..256 {
            let idx = 0x200 + p * 2;
            if pal[idx] != 0 || pal[idx+1] != 0 {
                let val = rd16(pal, idx);
                eprintln!("  First non-zero sprite pal[{}]: {:04X}", p, val);
                break;
            }
        }
        
        // Check VRAM at sprite tile area
        let vram: &[u8] = &emu.mem.vram[..];
        eprintln!("  VRAM[0x10000-0x1000F]: {:02X?}", &vram[0x10000..0x10010]);
    }
}
