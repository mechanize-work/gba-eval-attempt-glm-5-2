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
    
    // Step to 204600 (just before restart)
    for i in 0..204600 { gba_emu::emulator::step_one(); }
    
    // Trace 100 instructions
    for j in 0..200u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let mode = emu.cpu.cpsr & 0x1F;
        let mode_str = match mode { 0x10=>"USR",0x11=>"FIQ",0x12=>"IRQ",0x13=>"SVC",0x1F=>"SYS",_=>&"???" };
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let is_swi = if thumb { (instr & 0xFF00) == 0xDF00 } else { (instr >> 24) == 0xEF };
        
        if is_swi || pc < 0x04000000 || pc == 0x08000000 || pc == 0x080000C0 || j < 10 || j > 90 {
            let swi_info = if is_swi { 
                if thumb { format!(" SWI={}", instr & 0xFF) } 
                else { format!(" ARM_SWI=0x{:X}", instr & 0xFFFFFF) } 
            } else { String::new() };
            eprintln!("[{}] PC=0x{:08X} 0x{:X} {} {} DC=0x{:04X} LR=0x{:08X}{}",
                j, pc, instr, mode_str, if thumb {"T"} else {"A"}, dc, emu.cpu.r[14], swi_info);
        }
        
        gba_emu::emulator::step_one();
        
        // Check if we hit the EWRAM clear again (restart)
        if emu.cpu.r[15] == 0x08000190 {
            eprintln!("[{}] *** RESTARTED! Hit EWRAM clear ***", j+1);
            break;
        }
    }
}
