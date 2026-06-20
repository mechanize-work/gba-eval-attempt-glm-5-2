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
    for f in 0..3 { gba_emu::emulator::run_frame(); }
    
    eprintln!("After 3 rf: halted={} vbiw={} [15E0]={} cc={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait,
        emu.mem.read_word(0x030015E0), emu.cycle_count);
    
    // Run frame 4 using run_frame
    gba_emu::emulator::run_frame();
    
    eprintln!("After rf4: halted={} vbiw={} [15E0]={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait,
        emu.mem.read_word(0x030015E0));
    
    // But the manual loop after 3 rf works!
    // So run_frame must be doing something different.
    // Let me check: does run_frame's while loop even execute?
    // Maybe target_cycles wraps or something.
    
    // Run 3 more frames and check each one
    for f in 4..10 {
        let before_15e0 = emu.mem.read_word(0x030015E0);
        let before_halted = emu.cpu.halted;
        gba_emu::emulator::run_frame();
        let after_15e0 = emu.mem.read_word(0x030015E0);
        eprintln!("Frame {}: before halted={} [15E0]={} -> after halted={} [15E0]={}",
            f, before_halted, before_15e0, emu.cpu.halted, after_15e0);
    }
}
