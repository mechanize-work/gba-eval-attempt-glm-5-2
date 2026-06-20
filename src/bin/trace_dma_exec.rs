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
    
    // Step to 226525 (just before DMA enable)
    for i in 0..226525 { gba_emu::emulator::step_one(); }
    
    // Trace 20 instructions around the DMA enable
    for j in 0..20u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        let dc3_cnt = (emu.mem.io[0xDC] as u32) | ((emu.mem.io[0xDD] as u32) << 8)
            | ((emu.mem.io[0xDE] as u32) << 16) | ((emu.mem.io[0xDF] as u32) << 24);
        let dc3_sad = (emu.mem.io[0xD4] as u32) | ((emu.mem.io[0xD5] as u32) << 8)
            | ((emu.mem.io[0xD6] as u32) << 16) | ((emu.mem.io[0xD7] as u32) << 24);
        let dc3_dad = (emu.mem.io[0xD8] as u32) | ((emu.mem.io[0xD9] as u32) << 8)
            | ((emu.mem.io[0xDA] as u32) << 16) | ((emu.mem.io[0xDB] as u32) << 24);
        let dma3_cached = emu.mem.dma_cnt[3];
        let dma3_enabled = emu.dma.enabled[3];
        let pal_nz = emu.mem.palette.iter().filter(|&&b| b != 0).count();
        
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} DMA3CNT=0x{:08X} cached=0x{:08X} en={} SAD=0x{:08X} DAD=0x{:08X} pal_nz={}",
            j, pc, instr, dc3_cnt, dma3_cached, dma3_enabled, dc3_sad, dc3_dad, pal_nz);
        
        gba_emu::emulator::step_one();
    }
}
