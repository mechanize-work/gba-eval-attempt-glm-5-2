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
    
    // Step to 226430
    for i in 0..226430 { gba_emu::emulator::step_one(); }
    
    // Trace 30 instructions
    for j in 0..30u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let dc_before = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} DC=0x{:04X}",
            j, pc, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
            emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], dc_before);
        
        gba_emu::emulator::step_one();
        
        let dc_after = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc_after != dc_before {
            eprintln!("  *** DC changed: 0x{:04X} -> 0x{:04X} ***", dc_before, dc_after);
        }
    }
}
