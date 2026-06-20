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
    
    // Step until DISPCNT changes to 0x1000
    let mut last_dc = 0x0080u16;
    let mut changes = 0;
    
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != last_dc {
            eprintln!("[{}] DC 0x{:04X}->0x{:04X} scan={} PC=0x{:08X} halted={}",
                i, last_dc, dc, emu.current_scanline, emu.cpu.r[15], emu.cpu.halted);
            last_dc = dc;
            changes += 1;
            if changes >= 10 { break; }
        }
    }
}
