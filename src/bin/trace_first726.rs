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
    
    // Step until PC = 0x08000726 OR until halted
    for i in 0..500000u64 {
        let pc = emu.cpu.r[15];
        if pc == 0x08000726 {
            eprintln!("[{}] Reached 0x08000726", i);
            // Trace 80 instructions
            for j in 0..80u32 {
                let pc2 = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc2) as u32;
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let is_swi = (instr & 0xFF00) == 0xDF00;
                let is_bl = (instr & 0xF800) == 0xF000;
                let extra = if is_swi { format!(" SWI={}", instr & 0xFF) }
                    else if is_bl { " BL_HI".to_string() } else { String::new() };
                eprintln!("[{:2}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}{}",
                    j, pc2, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], extra);
                gba_emu::emulator::step_one();
                let dc2 = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                if dc2 != dc { eprintln!("  *** DC: 0x{:04X}->0x{:04X} ***", dc, dc2); }
                if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
            }
            return;
        }
        gba_emu::emulator::step_one();
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted before reaching 0x08000726, PC=0x{:08X}", i, emu.cpu.r[15]);
            return;
        }
    }
    eprintln!("Never reached 0x08000726");
}
