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
    
    // Track writes to 0x03007FFC
    let mut last_7ffc = emu.mem.read_word(0x03007FFC);
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    let mut last_3c18 = emu.mem.read_word(0x03003C18);
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        let v7ffc = emu.mem.read_word(0x03007FFC);
        let v15e0 = emu.mem.read_word(0x030015E0);
        let v3c18 = emu.mem.read_word(0x03003C18);
        
        if v7ffc != last_7ffc {
            eprintln!("[{}] [0x03007FFC] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}",
                i, last_7ffc, v7ffc, emu.cpu.r[15]);
            last_7ffc = v7ffc;
        }
        if v15e0 != last_15e0 {
            eprintln!("[{}] [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}",
                i, last_15e0, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
        }
        if v3c18 != last_3c18 {
            eprintln!("[{}] [0x03003C18] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}",
                i, last_3c18, v3c18, emu.cpu.r[15]);
            last_3c18 = v3c18;
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i, emu.cpu.r[15]);
            eprintln!("  [0x03007FFC] = 0x{:08X}", last_7ffc);
            eprintln!("  [0x030015E0] = 0x{:08X}", last_15e0);
            eprintln!("  [0x03003C18] = 0x{:08X}", last_3c18);
            break;
        }
    }
}
