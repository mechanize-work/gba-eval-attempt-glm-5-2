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
    
    // Track all writes to [0x030015E0]
    let mut last_val = emu.mem.read_word(0x030015E0);
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        let val = emu.mem.read_word(0x030015E0);
        if val != last_val {
            eprintln!("[{}] [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}",
                i, last_val, val, emu.cpu.r[15]);
            last_val = val;
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted", i);
            break;
        }
    }
    eprintln!("Final [0x030015E0] = 0x{:08X}", last_val);
}
