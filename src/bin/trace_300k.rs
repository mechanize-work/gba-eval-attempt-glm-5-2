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
    
    // Track all writes to DISPCNT and all SWI calls
    let mut last_dc = 0x0080u16;
    
    for i in 0..500_000u64 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let is_swi = (instr & 0xFF00) == 0xDF00;
        
        gba_emu::emulator::step_one();
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != last_dc {
            eprintln!("[{}] DISPCNT: 0x{:04X} -> 0x{:04X} at PC=0x{:08X} (instr=0x{:04X})",
                i, last_dc, dc, pc, instr);
            last_dc = dc;
        }
        
        if is_swi {
            eprintln!("[{}] SWI {} at PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X}",
                i, instr & 0xFF, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
        }
        
        if emu.cpu.halted && i > 226000 {
            // Check if it's VBlankIntrWait
            if is_swi || (instr & 0xFF00) == 0xDF00 {
                eprintln!("[{}] VBlankIntrWait halt", i);
            }
            break;
        }
    }
    eprintln!("Final DISPCNT: 0x{:04X}", last_dc);
}
