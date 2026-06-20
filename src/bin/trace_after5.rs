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
    
    // Step until exit #5 (step ~204639)
    let mut in_memset = true;
    let mut exit_count = 0u32;
    
    for i in 0..300_000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        
        let was_in = in_memset;
        in_memset = pc >= 0x08000186 && pc <= 0x08000196;
        
        if was_in && !in_memset {
            exit_count += 1;
            if exit_count >= 5 {
                eprintln!("Exit #5 at step {}", i);
                // Trace 100 instructions
                for j in 0..100u32 {
                    let pc2 = emu.cpu.r[15];
                    let thumb = emu.cpu.is_thumb();
                    let instr: u32 = if thumb { emu.mem.read_half(pc2) as u32 } else { emu.mem.read_word(pc2) };
                    
                    // Check for SWI
                    if thumb && (instr & 0xFF00) == 0xDF00 {
                        eprintln!("[{}] SWI {} at PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                            j, instr & 0xFF, pc2, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
                    }
                    
                    let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                    let halted = emu.cpu.halted;
                    let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
                    
                    if j < 30 || dispcnt != 0x0080 || halted || (instr & 0xFF00) == 0xDF00 {
                        eprintln!("[{:3}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X} DISPCNT=0x{:04X} halted={} IME={}",
                            j, pc2, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                            emu.cpu.r[13], emu.cpu.r[14], dispcnt, halted, ime);
                    }
                    
                    gba_emu::emulator::step_one();
                    
                    if emu.cpu.halted {
                        eprintln!("[{}] CPU HALTED!", j+1);
                        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
                        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
                        eprintln!("  IME={} IE=0x{:04X} IF=0x{:04X}", ime, ie, if_);
                        break;
                    }
                }
                break;
            }
            in_memset = true;
        }
    }
}
