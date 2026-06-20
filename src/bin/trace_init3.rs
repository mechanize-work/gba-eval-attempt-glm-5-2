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
    
    // Step until [0x030015E0] changes from 1 to 0
    let mut last_15e0 = 1u32;
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [0x030015E0] = {} -> {} at PC=0x{:08X}", i, last_15e0, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
            
            if v15e0 == 0 {
                eprintln!("  Init code should run now!");
                // Trace 100 instructions
                for j in 0..100u32 {
                    let pc = emu.cpu.r[15];
                    let instr = if emu.cpu.is_thumb() { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
                    let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                    let is_swi = (instr & 0xFF00) == 0xDF00 || (instr >> 24) == 0xEF;
                    
                    if is_swi || dc != 0x0080 || j < 10 || j > 90 {
                        let extra = if (instr & 0xFF00) == 0xDF00 { format!(" SWI={}", instr & 0xFF) } else { String::new() };
                        eprintln!("  [{:2}] PC=0x{:08X} 0x{:X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X}{}",
                            j, pc, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], extra);
                    }
                    
                    gba_emu::emulator::step_one();
                    
                    let dc2 = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                    if dc2 != dc { eprintln!("    *** DC: 0x{:04X}->0x{:04X} ***", dc, dc2); }
                    if emu.cpu.halted { eprintln!("  [{}] HALTED", j+1); break; }
                }
                break;
            }
        }
        if emu.cpu.halted { break; }
    }
}
