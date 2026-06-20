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
    
    // Track when PC reaches 0x080133C4
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        if pc == 0x080133C4 || pc == 0x080133C5 {
            eprintln!("[{}] Reached IRQ setup at PC=0x{:08X}", i, pc);
            // Check if 0x03007FFC gets written
            let before = emu.mem.read_word(0x03007FFC);
            for j in 0..30 {
                gba_emu::emulator::step_one();
            }
            let after = emu.mem.read_word(0x03007FFC);
            eprintln!("  [0x03007FFC] before=0x{:08X} after=0x{:08X}", before, after);
            return;
        }
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X} before reaching IRQ setup", i, pc);
            return;
        }
    }
    eprintln!("Never reached IRQ setup");
}
