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
    
    // Step to SWI 0x0C (step 226152) and execute it
    for i in 0..226153 {
        gba_emu::emulator::step_one();
    }
    
    // Now trace 100 instructions after the SWI
    for j in 0..100 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let halted = emu.cpu.halted;
        
        let is_swi = (instr & 0xFF00) == 0xDF00;
        let swi_info = if is_swi { format!(" *** SWI {} ***", instr & 0xFF) } else { String::new() };
        
        // Show SWI calls, DISPCNT changes, and key instructions
        if is_swi || dc != 0x0080 || halted || j < 20 || (j % 20 == 0) {
            eprintln!("[{:3}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} h={} R0={:08X} R1={:08X} R2={:08X} R3={:08X}{}",
                j, pc, instr, dc, halted,
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], swi_info);
        }
        
        gba_emu::emulator::step_one();
        
        if halted {
            eprintln!("[{}] HALTED", j+1);
            break;
        }
    }
}
