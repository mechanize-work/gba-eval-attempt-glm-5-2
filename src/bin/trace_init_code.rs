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
    
    // Run 2 frames to get to VBlank wait loop
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    // Step until CPU wakes from halt
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        if !emu.cpu.halted {
            // Trace 200 instructions looking for the init code at 0x08000726
            for j in 0..500 {
                let pc = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc) as u32;
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                
                // Show when we reach 0x08000726 or when DISPCNT changes
                if pc == 0x08000726 || pc == 0x08000728 || pc == 0x0800072A || 
                   pc == 0x0800072C || dc != 0x0080 || (instr & 0xFF00) == 0xDF00 ||
                   j < 5 {
                    eprintln!("[{:3}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} SP={:08X} LR={:08X}",
                        j, pc, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                        emu.cpu.r[4], emu.cpu.r[13], emu.cpu.r[14]);
                }
                
                gba_emu::emulator::step_one();
                
                if emu.cpu.halted {
                    eprintln!("[{:3}] HALTED", j+1);
                    break;
                }
                
                let dc2 = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                if dc2 != dc {
                    eprintln!("  *** DISPCNT changed: 0x{:04X} -> 0x{:04X} ***", dc, dc2);
                }
            }
            break;
        }
    }
}
