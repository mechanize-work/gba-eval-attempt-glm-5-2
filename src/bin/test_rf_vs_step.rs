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
    
    // Run 3 frames using run_frame
    for _ in 0..3 { gba_emu::emulator::run_frame(); }
    
    // State A: after 3 run_frame calls
    eprintln!("State A (after 3 rf): halted={} vbiw={} vb_occ={} [15E0]={} cc={} scan={} cis={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0), emu.cycle_count,
        emu.current_scanline, emu.cycle_in_scanline);
    
    // Now call emu.run_frame() directly (not through global function)
    emu.run_frame();
    
    eprintln!("After emu.run_frame(): halted={} vbiw={} [15E0]={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait,
        emu.mem.read_word(0x030015E0));
}
