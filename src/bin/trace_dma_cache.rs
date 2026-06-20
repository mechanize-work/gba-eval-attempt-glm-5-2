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
    
    // Step to 226525
    for i in 0..226525 { gba_emu::emulator::step_one(); }
    
    // Check io_write_half calls for DMA3 registers
    // The game writes to 0x040000DC (low) and 0x040000DE (high)
    // Let me track io[0xDC..0xDF] changes
    let mut last = [0u8; 4];
    last.copy_from_slice(&emu.mem.io[0xDC..0xE0]);
    
    for j in 0..30u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        gba_emu::emulator::step_one();
        
        let curr = &emu.mem.io[0xDC..0xE0];
        if curr != &last[..] {
            let cnt = (curr[0] as u32) | ((curr[1] as u32) << 8) | ((curr[2] as u32) << 16) | ((curr[3] as u32) << 24);
            let cached = emu.mem.dma_cnt[3];
            eprintln!("[{}] PC=0x{:08X} 0x{:04X} io[DC..DF]={:02X}{:02X}{:02X}{:02X} CNT=0x{:08X} cached=0x{:08X}",
                j, pc, instr, curr[3], curr[2], curr[1], curr[0], cnt, cached);
            last.copy_from_slice(curr);
        }
    }
}
