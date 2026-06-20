use std::fs;
use std::collections::HashSet;

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
    
    // Step through and track all unique PC ranges visited
    let mut visited: HashSet<u32> = HashSet::new();
    let mut last_pc = 0u32;
    
    for i in 0..3_000_000u32 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        
        // Track unique 256-byte ranges
        let range = pc & 0xFFFFFF00;
        if !visited.contains(&range) {
            visited.insert(range);
            eprintln!("[{}] New PC range: 0x{:08X} (PC=0x{:08X}) R0=0x{:08X} R1=0x{:08X}",
                i, range, pc, emu.cpu.r[0], emu.cpu.r[1]);
        }
        
        // Check for DISPCNT changes
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dispcnt != 0 {
            eprintln!("[{}] DISPCNT=0x{:04X} at PC=0x{:08X}", i, dispcnt, pc);
            break;
        }
        
        // Check for halt
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i, pc);
            break;
        }
    }
    eprintln!("Total unique PC ranges: {}", visited.len());
}
