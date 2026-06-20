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
    
    // Run 2 frames
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    // Step until [15E0] changes
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] 0x{:08X} -> 0x{:08X} at PC=0x{:08X}", i, last_15e0, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
            
            // Trace 500 instructions
            let mut last_dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            for j in 0..500u32 {
                let pc = emu.cpu.r[15];
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let v = emu.mem.read_word(0x030015E0);
                let thumb = emu.cpu.is_thumb();
                let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
                let is_swi = if thumb { (instr & 0xFF00) == 0xDF00 } else { (instr >> 24) == 0xEF };
                
                if dc != last_dc { eprintln!("[{}] DC 0x{:04X}->0x{:04X} PC=0x{:08X}", j, last_dc, dc, pc); last_dc = dc; }
                if v != last_15e0 { eprintln!("[{}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X}", j, last_15e0, v, pc); last_15e0 = v; }
                
                if pc == 0x0800070C || pc == 0x08000714 || pc == 0x08000716 || pc == 0x08000726 || is_swi {
                    eprintln!("[{}] PC=0x{:08X} 0x{:X} [15E0]=0x{:08X} R0={:08X} R3={:08X}",
                        j, pc, instr, v, emu.cpu.r[0], emu.cpu.r[3]);
                }
                
                gba_emu::emulator::step_one();
                if emu.cpu.halted { 
                    eprintln!("[{}] HALTED at PC=0x{:08X} vbiw={}", j+1, emu.cpu.r[15], emu.cpu.vblank_intr_wait);
                    break; 
                }
            }
            break;
        }
    }
}
