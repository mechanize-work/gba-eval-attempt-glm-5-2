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
    
    // Run 2 frames using run_frame (gets stuck)
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    eprintln!("After 2 run_frame: halted={} vbiw={} vb_occ={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0));
    
    // Check: does step_one trigger VBlankIntrWait?
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let v = emu.mem.read_word(0x030015E0);
        if v != last_15e0 {
            eprintln!("[step {}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X}", i, last_15e0, v, emu.cpu.r[15]);
            last_15e0 = v;
        }
        if !emu.cpu.halted && i > 1000 {
            eprintln!("[step {}] woke! PC=0x{:08X}", i, emu.cpu.r[15]);
            break;
        }
    }
    
    // Now try run_frame again
    eprintln!("\nBefore run_frame: halted={} vbiw={} vb_occ={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred);
    gba_emu::emulator::run_frame();
    eprintln!("After run_frame: halted={} vbiw={} vb_occ={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0));
}
