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
    
    // Step until we reach 0x08000726
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        if emu.cpu.r[15] == 0x08000726 {
            eprintln!("[{}] Reached init code at 0x08000726", i);
            eprintln!("  R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}",
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[13], emu.cpu.r[14]);
            
            // Trace 200 instructions from here
            for j in 0..500u32 {
                let pc = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc) as u32;
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let is_swi = (instr & 0xFF00) == 0xDF00;
                let is_bl = (instr & 0xF800) == 0xF000; // BL hi
                
                if is_swi || is_bl || dc != 0x0080 || j < 20 || j > 480 {
                    let swi = if is_swi { format!(" SWI={}", instr & 0xFF) } else { String::new() };
                    eprintln!("[{:3}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}{}",
                        j, pc, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], swi);
                }
                
                gba_emu::emulator::step_one();
                
                let dc2 = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                if dc2 != dc {
                    eprintln!("  *** DISPCNT changed: 0x{:04X} -> 0x{:04X} ***", dc, dc2);
                }
                
                if emu.cpu.halted {
                    eprintln!("[{}] HALTED", j+1);
                    break;
                }
            }
            break;
        }
    }
}
