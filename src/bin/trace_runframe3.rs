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
    
    // Run 2 frames using run_frame
    for f in 0..2 {
        gba_emu::emulator::run_frame();
        eprintln!("Frame {}: halted={} vbiw={} PC=0x{:08X} cc={} [15E0]=0x{:08X}",
            f, emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cpu.r[15], emu.cycle_count,
            emu.mem.read_word(0x030015E0));
    }
    
    // Now run frame 2 using run_frame and check
    gba_emu::emulator::run_frame();
    eprintln!("Frame 2: halted={} vbiw={} PC=0x{:08X} cc={} [15E0]=0x{:08X} DC=0x{:04X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cpu.r[15], emu.cycle_count,
        emu.mem.read_word(0x030015E0),
        (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
    
    // Run more frames
    for f in 3..10 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        eprintln!("Frame {}: halted={} PC=0x{:08X} [15E0]=0x{:08X} DC=0x{:04X}",
            f, emu.cpu.halted, emu.cpu.r[15], emu.mem.read_word(0x030015E0), dc);
    }
}
