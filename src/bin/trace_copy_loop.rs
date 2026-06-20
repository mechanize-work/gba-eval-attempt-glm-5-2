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
    
    // Step to 210530 (just before the copy)
    for i in 0..210530 { gba_emu::emulator::step_one(); }
    
    // Trace 20 instructions
    for j in 0..20 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} SP={:08X} LR={:08X}",
            j, pc, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
            emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7], emu.cpu.r[13], emu.cpu.r[14]);
        gba_emu::emulator::step_one();
    }
}
