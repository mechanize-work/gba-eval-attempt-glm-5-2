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
    
    // Track all writes to DISPCNT (0x04000000)
    let mut last_dc = 0x0080u16;
    
    for i in 0..500_000u64 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        // Check if this instruction might write to DISPCNT
        // STR/STRH to address containing 0x04000000
        // For simplicity, just check DISPCNT after each instruction
        gba_emu::emulator::step_one();
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != last_dc {
            eprintln!("[{}] DISPCNT changed: 0x{:04X} -> 0x{:04X} at PC=0x{:08X} (instr=0x{:X})",
                i, last_dc, dc, pc, instr);
            last_dc = dc;
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i, emu.cpu.r[15]);
            break;
        }
    }
    eprintln!("Final DISPCNT: 0x{:04X}", last_dc);
}
