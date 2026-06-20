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
    
    // Check state
    let vram_nonzero = emu.mem.vram.iter().filter(|&&b| b != 0).count();
    let pal_nonzero = emu.mem.palette.iter().filter(|&&b| b != 0).count();
    eprintln!("VRAM non-zero: {}/{}", vram_nonzero, emu.mem.vram.len());
    eprintln!("Palette non-zero: {}/{}", pal_nonzero, emu.mem.palette.len());
    eprintln!("DISPCNT=0x{:04X} snap=0x{:04X}", 
        (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8),
        emu.ppu.snap_dispcnt);
    
    // Check DMA registers
    for i in 0..4 {
        let cnt_off = 0xB8 + i * 0x0C;
        let cnt = (emu.mem.io[cnt_off] as u32) | ((emu.mem.io[cnt_off+1] as u32) << 8)
            | ((emu.mem.io[cnt_off+2] as u32) << 16) | ((emu.mem.io[cnt_off+3] as u32) << 24);
        let sad_off = 0xB0 + i * 0x0C;
        let sad = (emu.mem.io[sad_off] as u32) | ((emu.mem.io[sad_off+1] as u32) << 8)
            | ((emu.mem.io[sad_off+2] as u32) << 16) | ((emu.mem.io[sad_off+3] as u32) << 24);
        let dad_off = 0xB4 + i * 0x0C;
        let dad = (emu.mem.io[dad_off] as u32) | ((emu.mem.io[dad_off+1] as u32) << 8)
            | ((emu.mem.io[dad_off+2] as u32) << 16) | ((emu.mem.io[dad_off+3] as u32) << 24);
        eprintln!("DMA{}: SAD=0x{:08X} DAD=0x{:08X} CNT=0x{:08X} enabled={}", 
            i, sad, dad, cnt, cnt & 0x80000000 != 0);
    }
}
