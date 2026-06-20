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
    
    // Step until we first reach the memset loop
    for i in 0..1_000_000u32 {
        gba_emu::emulator::step_one();
        if emu.cpu.r[15] == 0x08000190 {
            eprintln!("First reached memset at step {}, R0=0x{:08X} R1=0x{:08X}", 
                i, emu.cpu.r[0], emu.cpu.r[1]);
            
            // Now track R1 over time to see if it's decreasing
            let mut last_r1 = emu.cpu.r[1];
            for j in 0..500_000u32 {
                gba_emu::emulator::step_one();
                let pc = emu.cpu.r[15];
                let r1 = emu.cpu.r[1];
                
                // Print when R1 changes significantly or PC exits the loop
                if pc < 0x08000190 || pc > 0x08000196 {
                    eprintln!("[{}] Exited loop! PC=0x{:08X} R0=0x{:08X} R1=0x{:08X}", 
                        j, pc, emu.cpu.r[0], r1);
                    
                    // Trace a few more instructions
                    for k in 0..30 {
                        let pc2 = emu.cpu.r[15];
                        let instr = emu.mem.read_half(pc2) as u32;
                        gba_emu::emulator::step_one();
                        eprintln!("  [{}+{}] PC=0x{:08X} 0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X}",
                            j, k, pc2, instr, 
                            emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[4]);
                    }
                    break;
                }
                
                if j % 50000 == 0 {
                    eprintln!("[{}] PC=0x{:08X} R0=0x{:08X} R1=0x{:08X}", j, pc, emu.cpu.r[0], r1);
                }
            }
            break;
        }
    }
}
