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
    
    // Track all DMA enable transitions
    let mut last_dma = [0u32; 4];
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        for ch in 0..4 {
            let cnt_off = 0xB8 + ch * 0x0C;
            let cnt = (emu.mem.io[cnt_off] as u32) | ((emu.mem.io[cnt_off+1] as u32) << 8)
                | ((emu.mem.io[cnt_off+2] as u32) << 16) | ((emu.mem.io[cnt_off+3] as u32) << 24);
            let was_enabled = last_dma[ch] & 0x80000000 != 0;
            let is_enabled = cnt & 0x80000000 != 0;
            if is_enabled && !was_enabled {
                let sad_off = 0xB0 + ch * 0x0C;
                let sad = (emu.mem.io[sad_off] as u32) | ((emu.mem.io[sad_off+1] as u32) << 8)
                    | ((emu.mem.io[sad_off+2] as u32) << 16) | ((emu.mem.io[sad_off+3] as u32) << 24);
                let dad_off = 0xB4 + ch * 0x0C;
                let dad = (emu.mem.io[dad_off] as u32) | ((emu.mem.io[dad_off+1] as u32) << 8)
                    | ((emu.mem.io[dad_off+2] as u32) << 16) | ((emu.mem.io[dad_off+3] as u32) << 24);
                eprintln!("[{}] DMA{} enable: SAD=0x{:08X} DAD=0x{:08X} CNT=0x{:08X} PC=0x{:08X}",
                    i, ch, sad, dad, cnt, emu.cpu.r[15]);
            }
            last_dma[ch] = cnt;
        }
        
        // Check palette/VRAM for changes
        if i == 226530 || i == 226550 || i == 226600 {
            let pal_nz = emu.mem.palette.iter().filter(|&&b| b != 0).count();
            let vram_nz = emu.mem.vram.iter().filter(|&&b| b != 0).count();
            eprintln!("[{}] pal_nz={} vram_nz={}", i, pal_nz, vram_nz);
        }
    }
}
