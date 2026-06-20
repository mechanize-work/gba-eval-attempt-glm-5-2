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
    
    // Step to 225075 (IRQ setup)
    for i in 0..225075 { gba_emu::emulator::step_one(); }
    
    // Trace 50 instructions from 0x080133C4
    for j in 0..50 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let r2 = emu.cpu.r[2];
        let r3 = emu.cpu.r[3];
        
        // Before executing, show state
        if pc >= 0x08013400 && pc <= 0x08013410 {
            let addr = if r3 >= 0x03000000 && r3 < 0x03008000 {
                r3
            } else { 0 };
            eprintln!("[{}] PC=0x{:08X} 0x{:04X} R2=0x{:08X} R3=0x{:08X} [R3]=0x{:08X}",
                j, pc, instr, r2, r3, if addr > 0 { emu.mem.read_word(addr) } else { 0 });
        }
        
        gba_emu::emulator::step_one();
        
        // After executing, check if 0x03007FFC changed
        if pc == 0x0801340A {
            let val = emu.mem.read_word(0x03007FFC);
            eprintln!("  After STR: [0x03007FFC] = 0x{:08X}", val);
        }
    }
}
