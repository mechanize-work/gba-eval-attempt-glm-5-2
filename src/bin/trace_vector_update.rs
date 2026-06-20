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
    
    // Track changes to [0x03003E5C] (VBlank vector table entry)
    let mut last_vb = emu.mem.read_word(0x03003E5C);
    
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        
        let vb = emu.mem.read_word(0x03003E5C);
        if vb != last_vb {
            eprintln!("[{}] [0x03003E5C] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}",
                i, last_vb, vb, emu.cpu.r[15]);
            last_vb = vb;
        }
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != 0x0080 {
            eprintln!("[{}] DISPCNT=0x{:04X} at PC=0x{:08X}", i, dc, emu.cpu.r[15]);
            break;
        }
        
        if emu.cpu.halted && i > 226000 {
            // Continue past halt - the IRQ should wake it up
        }
    }
    eprintln!("Final [0x03003E5C] = 0x{:08X}", last_vb);
}
