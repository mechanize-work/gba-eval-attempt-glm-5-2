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
    
    // Step to just before SWI 0x0C (step 226152)
    for i in 0..226150 {
        gba_emu::emulator::step_one();
    }
    
    eprintln!("Before SWI 0x0C:");
    eprintln!("  R0=0x{:08X} R1=0x{:08X} R2=0x{:08X}", emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
    eprintln!("  PC=0x{:08X}", emu.cpu.r[15]);
    
    // Check IWRAM before
    let before = emu.mem.read_word(0x03000000);
    eprintln!("  IWRAM[0] before = 0x{:08X}", before);
    
    // Check what's in ROM at the source address
    let src = emu.cpu.r[0];
    let rom_val = emu.mem.read_word(src & !3);
    eprintln!("  ROM[0x{:08X}] = 0x{:08X}", src & !3, rom_val);
    
    // Execute the SWI
    gba_emu::emulator::step_one();
    gba_emu::emulator::step_one(); // SWI is 2 instructions in THUMB? No, it's one
    
    // Actually, step_one executes one instruction. SWI is one instruction.
    // But we're at step 226150, need to get to 226152
    gba_emu::emulator::step_one();
    
    eprintln!("\nAfter SWI 0x0C:");
    eprintln!("  R0=0x{:08X} R1=0x{:08X} R2=0x{:08X}", emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
    eprintln!("  PC=0x{:08X}", emu.cpu.r[15]);
    
    // Check IWRAM after
    for i in 0..10 {
        let addr = 0x03000000 + i * 4;
        let val = emu.mem.read_word(addr);
        eprintln!("  IWRAM[0x{:08X}] = 0x{:08X}", addr, val);
    }
}
