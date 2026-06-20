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
    
    // Step to 0x08013420
    for i in 0..226250 { gba_emu::emulator::step_one(); }
    
    // Trace 200 instructions, tracking [0x030015E0] and DISPCNT
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    let mut last_dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    let mut last_vb = emu.mem.read_word(0x03003E5C);
    
    for j in 0..500u32 {
        let pc = emu.cpu.r[15];
        let instr = if emu.cpu.is_thumb() { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let v15e0 = emu.mem.read_word(0x030015E0);
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let vb = emu.mem.read_word(0x03003E5C);
        
        // Show changes
        if v15e0 != last_15e0 {
            eprintln!("[{}] *** [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X} ***", j, last_15e0, v15e0, pc);
            last_15e0 = v15e0;
        }
        if dc != last_dc {
            eprintln!("[{}] *** DISPCNT = 0x{:04X} -> 0x{:04X} at PC=0x{:08X} ***", j, last_dc, dc, pc);
            last_dc = dc;
        }
        if vb != last_vb {
            eprintln!("[{}] *** [0x03003E5C] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X} ***", j, last_vb, vb, pc);
            last_vb = vb;
        }
        
        // Show all instructions in 0x080134xx range
        if pc >= 0x08013400 && pc < 0x08013600 {
            let is_swi = (instr & 0xFF00) == 0xDF00;
            let extra = if is_swi { format!(" SWI={}", instr & 0xFF) } else { String::new() };
            eprintln!("[{:3}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}{}",
                j, pc, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], extra);
        }
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
    }
}
