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
    
    // Run 2 frames using the GLOBAL run_frame (same as harness)
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    eprintln!("After 2 global run_frame: halted={} vbiw={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.mem.read_word(0x030015E0));
    
    // Run frame 3 using the global run_frame
    gba_emu::emulator::run_frame();
    eprintln!("After frame 3 (global): halted={} vbiw={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.mem.read_word(0x030015E0));
    
    // Now run frame 4 using step_one (which works)
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let v = emu.mem.read_word(0x030015E0);
        if v != last_15e0 {
            eprintln!("[step] [15E0] 0x{:08X}->0x{:08X} at PC=0x{:08X}", last_15e0, v, emu.cpu.r[15]);
            last_15e0 = v;
        }
        if !emu.cpu.halted && i > 1000 {
            eprintln!("[step] CPU woke at PC=0x{:08X} [15E0]=0x{:08X}", emu.cpu.r[15], v);
            break;
        }
    }
}
