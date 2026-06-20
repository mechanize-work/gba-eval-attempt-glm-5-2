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
    
    // Track writes to IWRAM 0x03007E34-0x03007E54 (first 32 bytes of CpuFastSet source)
    let watch_start = 0x7E34usize;
    let watch_end = 0x7E54usize;
    let mut last_vals: Vec<u8> = emu.mem.iwram[watch_start..watch_end].to_vec();
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        for j in 0..(watch_end - watch_start) {
            if emu.mem.iwram[watch_start + j] != last_vals[j] {
                let addr = 0x03000000 + watch_start + j;
                eprintln!("[{}] [0x{:08X}] 0x{:02X} -> 0x{:02X} at PC=0x{:08X}",
                    i, addr, last_vals[j], emu.mem.iwram[watch_start + j], emu.cpu.r[15]);
                last_vals[j] = emu.mem.iwram[watch_start + j];
            }
        }
        
        if emu.cpu.halted && i > 100000 {
            // Continue past halt
        }
        
        // Stop after CpuFastSet (SWI 0x0C) to BIOS
        let pc = emu.cpu.r[15];
        if pc == 0x08000734 || (pc >= 0x08000730 && pc <= 0x08000740) {
            // Check IWRAM state
            eprintln!("\nIWRAM at 0x03007E34 after step {}:", i);
            for k in 0..8 {
                let val = emu.mem.read_word(0x03007E34 + k * 4);
                eprintln!("  [0x{:08X}] = 0x{:08X}", 0x03007E34 + k * 4, val);
            }
        }
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("DISPLAY ACTIVE at step {}!", i);
            break;
        }
    }
}
