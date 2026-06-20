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
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    eprintln!("State: halted={} vbiw={} vb_occ={} cc={} scan={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.cycle_count, emu.current_scanline, emu.mem.read_word(0x030015E0));
    
    // Now run frame 3 using run_frame
    gba_emu::emulator::run_frame();
    eprintln!("After run_frame: halted={} vbiw={} vb_occ={} cc={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.cycle_count, emu.mem.read_word(0x030015E0));
    
    // The key question: does run_frame's while loop see vb_occ=true?
    // Let me add a counter inside check_and_handle_interrupts
    // by checking the state before and after
}
