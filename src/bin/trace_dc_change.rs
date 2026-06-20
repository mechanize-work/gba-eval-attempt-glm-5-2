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
    
    // Track ALL writes to DISPCNT (0x04000000-0x04000001)
    let mut last_dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    
    for i in 0..2_000_000u64 {
        gba_emu::emulator::step_one();
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != last_dc {
            eprintln!("[{}] DISPCNT 0x{:04X} -> 0x{:04X} at PC=0x{:08X}",
                i, last_dc, dc, emu.cpu.r[15]);
            last_dc = dc;
        }
        
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("  DISPLAY ACTIVE at step {}!", i);
            break;
        }
    }
    eprintln!("Final DISPCNT: 0x{:04X}", last_dc);
}
