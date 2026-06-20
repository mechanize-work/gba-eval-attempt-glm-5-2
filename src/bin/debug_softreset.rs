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
    
    for frame in 0..10 {
        let emu = gba_emu::emulator::get_emu();
        let iwram: &[u8] = &emu.mem.iwram[..];
        
        // Check SoftReset flag before running frame
        let flag = iwram[0x7FFA];
        let pc = emu.cpu.r[15];
        
        gba_emu::emulator::run_frame();
        
        let emu = gba_emu::emulator::get_emu();
        let iwram: &[u8] = &emu.mem.iwram[..];
        let flag_after = iwram[0x7FFA];
        
        eprintln!("Frame {}: PC=0x{:08X} -> 0x{:08X} SoftReset_flag: 0x{:02X} -> 0x{:02X} EWRAM[0]={:02X}",
            frame, pc, emu.cpu.r[15], flag, flag_after, emu.mem.ewram[0]);
    }
}
