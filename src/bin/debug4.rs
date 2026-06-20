// Debug: trace loop exit
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
    
    let mut last_pc = 0u32;
    let mut loop_count = 0u32;
    
    for i in 0..5_000_000u32 {
        gba_emu::emulator::step_one();
        
        let pc = emu.cpu.r[15];
        if pc == last_pc {
            loop_count += 1;
        } else {
            loop_count = 0;
            last_pc = pc;
        }
        
        // Check if we've exited the loop area
        if pc < 0x080001A0 || pc > 0x080001D0 {
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            eprintln!("[{}] Exited loop! PC=0x{:08X} DISPCNT=0x{:04X} cycles={}", 
                i, pc, dispcnt, emu.cycle_count);
            
            // Trace a few more instructions
            for j in 0..20 {
                let pc = emu.cpu.r[15];
                let thumb = emu.cpu.is_thumb();
                let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
                gba_emu::emulator::step_one();
                let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                eprintln!("  [{}+{}] PC=0x{:08X} instr=0x{:X} DISPCNT=0x{:04X} halted={}",
                    i, j, pc, instr, dispcnt, emu.cpu.halted);
            }
            break;
        }
        
        if i % 1_000_000 == 0 {
            eprintln!("[{}] Still in loop, PC=0x{:08X} R1=0x{:08X} cycles={}", 
                i, pc, emu.cpu.r[1], emu.cycle_count);
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i, pc);
            break;
        }
    }
}
