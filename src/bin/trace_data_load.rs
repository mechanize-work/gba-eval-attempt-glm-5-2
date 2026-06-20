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
    
    // Track writes to VRAM (0x06000000-0x06017FFF)
    let mut vram_writes = 0u32;
    let mut first_vram_write_step = 0u64;
    
    // Track DMA enable writes (0x040000B8-0x040000DC)
    let mut dma_writes = 0u32;
    
    // Track writes to palette (0x05000000)
    let mut pal_writes = 0u32;
    
    for i in 0..500000u64 {
        let pc_before = emu.cpu.r[15];
        gba_emu::emulator::step_one();
        
        // Check VRAM for changes (sample first 1024 bytes)
        if vram_writes == 0 {
            for j in 0..1024 {
                if emu.mem.vram[j] != 0 {
                    vram_writes += 1;
                    if first_vram_write_step == 0 {
                        first_vram_write_step = i;
                        eprintln!("[{}] First VRAM write at PC=0x{:08X}", i, pc_before);
                    }
                    break;
                }
            }
        }
        
        // Check DMA registers
        let dma3_cnt = (emu.mem.io[0xDC] as u32) | ((emu.mem.io[0xDD] as u32) << 8) |
                       ((emu.mem.io[0xDE] as u32) << 16) | ((emu.mem.io[0xDF] as u32) << 24);
        if dma3_cnt & 0x80000000 != 0 && dma_writes < 3 {
            dma_writes += 1;
            let sad = (emu.mem.io[0xD4] as u32) | ((emu.mem.io[0xD5] as u32) << 8) |
                       ((emu.mem.io[0xD6] as u32) << 16) | ((emu.mem.io[0xD7] as u32) << 24);
            let dad = (emu.mem.io[0xD8] as u32) | ((emu.mem.io[0xD9] as u32) << 8) |
                       ((emu.mem.io[0xDA] as u32) << 16) | ((emu.mem.io[0xDB] as u32) << 24);
            eprintln!("[{}] DMA3 enabled: SAD=0x{:08X} DAD=0x{:08X} CNT=0x{:08X} PC=0x{:08X}",
                i, sad, dad, dma3_cnt, pc_before);
        }
        
        // Check palette
        if pal_writes == 0 {
            for j in 0..32 {
                if j > 1 && emu.mem.palette[j] != 0 {
                    pal_writes += 1;
                    eprintln!("[{}] First palette write at PC=0x{:08X} pal[{}]=0x{:02X}",
                        i, pc_before, j, emu.mem.palette[j]);
                    break;
                }
            }
        }
    }
    
    eprintln!("\nSummary:");
    eprintln!("  First VRAM write: step {} (found={})", first_vram_write_step, vram_writes > 0);
    eprintln!("  DMA enables: {}", dma_writes);
    eprintln!("  Palette writes: {}", pal_writes);
    
    // Count non-zero VRAM
    let vram_nonzero = emu.mem.vram.iter().filter(|&&b| b != 0).count();
    eprintln!("  VRAM non-zero: {}/{}", vram_nonzero, emu.mem.vram.len());
}
