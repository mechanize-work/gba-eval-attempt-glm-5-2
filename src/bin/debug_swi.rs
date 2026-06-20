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
    
    // Run frame by frame, checking state between frames
    for frame in 0..5 {
        let emu = gba_emu::emulator::get_emu();
        let io: &[u8] = &emu.mem.io[..];
        let dispcnt = rd16(io, 0);
        eprintln!("Before frame {}: PC={:08X} DISPCNT={:04X} IE={:04X} IF={:04X} IME={:04X} halted={} vbwait={}",
            frame, emu.cpu.r[15], dispcnt, rd16(io, 0x200), rd16(io, 0x202), rd16(io, 0x208),
            emu.cpu.halted, emu.cpu.vblank_intr_wait);
        
        gba_emu::emulator::run_frame();
        
        let emu = gba_emu::emulator::get_emu();
        let io: &[u8] = &emu.mem.io[..];
        let dispcnt = rd16(io, 0);
        eprintln!("After  frame {}: PC={:08X} DISPCNT={:04X} IE={:04X} IF={:04X} IME={:04X} halted={} vbwait={}",
            frame, emu.cpu.r[15], dispcnt, rd16(io, 0x200), rd16(io, 0x202), rd16(io, 0x208),
            emu.cpu.halted, emu.cpu.vblank_intr_wait);
    }
}
