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
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    eprintln!("After 2 rf: halted={} vbiw={} [15E0]=0x{:08X} cc={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.mem.read_word(0x030015E0), emu.cycle_count);
    
    // Run 300000 step_one calls (equivalent to ~1 frame of halt)
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let v = emu.mem.read_word(0x030015E0);
        if v != last_15e0 {
            eprintln!("[step {}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X} halted={}", i, last_15e0, v, emu.cpu.r[15], emu.cpu.halted);
            last_15e0 = v;
        }
        if !emu.cpu.halted && i > 1000 {
            // CPU woke up, let it run a bit then stop
            if i > 2000 { break; }
        }
    }
    eprintln!("After steps: halted={} vbiw={} [15E0]=0x{:08X} cc={} PC=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, last_15e0, emu.cycle_count, emu.cpu.r[15]);
    
    // Now try run_frame
    gba_emu::emulator::run_frame();
    eprintln!("After rf: halted={} vbiw={} [15E0]=0x{:08X} PC=0x{:08X} DC=0x{:04X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.mem.read_word(0x030015E0),
        emu.cpu.r[15], (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
}
