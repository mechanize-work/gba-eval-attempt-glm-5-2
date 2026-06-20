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
    
    // Run 5 frames
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Now trace 100 instructions, showing what happens around VBlankIntrWait
    for i in 0..100 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let halted = emu.cpu.halted;
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        
        // Show SWI and key instructions
        if (instr & 0xFF00) == 0xDF00 || halted || dc != 0x0080 || i < 5 || i > 90 {
            eprintln!("[{:3}] PC=0x{:08X} 0x{:04X} h={} DC=0x{:04X} IF=0x{:04X} IE=0x{:04X} IME={} R0={:08X} R1={:08X}",
                i, pc, instr, halted, dc, if_, ie, ime, emu.cpu.r[0], emu.cpu.r[1]);
        }
        
        gba_emu::emulator::step_one();
    }
}
