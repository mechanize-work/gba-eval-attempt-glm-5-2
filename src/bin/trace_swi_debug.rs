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
    
    // Step to 226150
    for i in 0..226150 {
        gba_emu::emulator::step_one();
    }
    
    // Trace 5 instructions around the SWI
    for j in 0..5 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let is_swi = (instr & 0xFF00) == 0xDF00;
        
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} {} R0={:08X} R1={:08X} R2={:08X}",
            j, pc, instr, if is_swi {format!("*** SWI {} ***", instr & 0xFF)} else {String::new()},
            emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
        
        // Before stepping, check IWRAM
        if is_swi {
            let before = emu.mem.read_word(0x03000000);
            eprintln!("  IWRAM[0] before SWI = 0x{:08X}", before);
        }
        
        gba_emu::emulator::step_one();
        
        if is_swi {
            let after = emu.mem.read_word(0x03000000);
            eprintln!("  IWRAM[0] after SWI = 0x{:08X}", after);
            eprintln!("  R0={:08X} R1={:08X} R2={:08X} PC={:08X}", 
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[15]);
        }
    }
}
