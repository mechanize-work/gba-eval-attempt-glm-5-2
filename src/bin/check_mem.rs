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
    
    // Step to 282190
    for i in 0..282190 {
        gba_emu::emulator::step_one();
    }
    
    // Check memory at 0x0300253C
    let addr = 0x0300253C;
    let val = emu.mem.read_word(addr);
    eprintln!("[0x{:08X}] = 0x{:08X}", addr, val);
    
    // Also check R0 and R6
    eprintln!("R0=0x{:08X} R6=0x{:08X}", emu.cpu.r[0], emu.cpu.r[6]);
    eprintln!("R6+R0 = 0x{:08X}", emu.cpu.r[6].wrapping_add(emu.cpu.r[0]));
    
    // Read from R6+R0
    let addr2 = emu.cpu.r[6].wrapping_add(emu.cpu.r[0]);
    let val2 = emu.mem.read_word(addr2);
    eprintln!("[R6+R0]=[0x{:08X}] = 0x{:08X}", addr2, val2);
}
