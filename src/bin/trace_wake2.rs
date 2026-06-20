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
    
    // Step until halted (VBlankIntrWait)
    let mut halt_step = 0;
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        if emu.cpu.halted {
            halt_step = i;
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i, emu.cpu.r[15]);
            break;
        }
    }
    
    // Now step until wake-up
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        if !emu.cpu.halted {
            eprintln!("[{}] CPU woke up! PC=0x{:08X}", halt_step + i + 1, emu.cpu.r[15]);
            
            // Trace 100 instructions looking for 0x08000726 or DISPCNT change
            for j in 0..500u32 {
                let pc = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc) as u32;
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let is_swi = (instr & 0xFF00) == 0xDF00;
                
                if pc == 0x08000726 || dc != 0x0080 || is_swi || j < 10 || j > 480 {
                    let extra = if is_swi { format!(" SWI={}", instr & 0xFF) } else { String::new() };
                    eprintln!("[{:3}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}{}",
                        j, pc, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], extra);
                }
                
                gba_emu::emulator::step_one();
                
                let dc2 = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                if dc2 != dc {
                    eprintln!("  *** DISPCNT: 0x{:04X} -> 0x{:04X} ***", dc, dc2);
                }
                if emu.cpu.halted {
                    eprintln!("[{:3}] HALTED again at PC=0x{:08X}", j+1, emu.cpu.r[15]);
                    break;
                }
            }
            break;
        }
    }
}
