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
    
    // Run 1 frame to get to VBlankIntrWait
    gba_emu::emulator::run_frame();
    eprintln!("After frame 0: PC=0x{:08X} halted={} vbiw={} vb_occ={}",
        emu.cpu.r[15], emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred);
    
    // Run frame 1 (should wake from VBlankIntrWait)
    gba_emu::emulator::run_frame();
    eprintln!("After frame 1: PC=0x{:08X} halted={} vbiw={} vb_occ={} [15E0]=0x{:08X}",
        emu.cpu.r[15], emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0));
    
    // Run frame 2
    gba_emu::emulator::run_frame();
    eprintln!("After frame 2: PC=0x{:08X} halted={} vbiw={} vb_occ={} [15E0]=0x{:08X} DC=0x{:04X}",
        emu.cpu.r[15], emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0),
        (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
}
