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
    
    let mut last_15e0 = 0u32;
    let mut last_dc = 0x0080u16;
    
    for i in 0..1_000_000u64 {
        gba_emu::emulator::step_one();
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        
        if v15e0 != last_15e0 {
            eprintln!("[{}] [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}", i, last_15e0, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
        }
        if dc != last_dc {
            eprintln!("[{}] DISPCNT = 0x{:04X} -> 0x{:04X} at PC=0x{:08X}", i, last_dc, dc, emu.cpu.r[15]);
            last_dc = dc;
        }
        
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("DISPLAY ACTIVE at step {}!", i);
            break;
        }
    }
    eprintln!("Final: [15E0]=0x{:08X} DC=0x{:04X}", last_15e0, last_dc);
}
