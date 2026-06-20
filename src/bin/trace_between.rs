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
    
    // Run 5 frames to get to VBlankIntrWait loop
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Now trace ALL instructions between two VBlankIntrWait calls
    let mut swi_count = 0;
    for i in 0..500u64 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        
        if (instr & 0xFF00) == 0xDF00 {
            swi_count += 1;
            eprintln!("[{}] SWI {} at PC=0x{:08X}", i, instr & 0xFF, pc);
            if swi_count >= 3 { break; }
        }
        
        // Log all instructions (there should be few between VBIW calls)
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} SP={:08X} LR={:08X} halted={}",
            i, pc, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
            emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7], emu.cpu.r[13], emu.cpu.r[14], emu.cpu.halted);
        
        gba_emu::emulator::step_one();
    }
}
