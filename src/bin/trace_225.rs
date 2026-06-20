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
    
    // Step to 225100
    for i in 0..225100 { gba_emu::emulator::step_one(); }
    
    // Trace until restart
    for j in 0..300u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let mode = emu.cpu.cpsr & 0x1F;
        let mode_str = match mode { 0x10=>"USR",0x11=>"FIQ",0x12=>"IRQ",0x13=>"SVC",0x1F=>"SYS",_=>"??" };
        let is_swi = if thumb { (instr & 0xFF00) == 0xDF00 } else { (instr >> 24) == 0xEF };
        
        let swi = if is_swi {
            if thumb { format!(" SWI={}", instr & 0xFF) }
            else { format!(" ARM_SWI=0x{:X}", instr & 0xFFFFFF) }
        } else { String::new() };
        
        eprintln!("[225100+{}] PC=0x{:08X} 0x{:X} {}{} LR=0x{:08X} R0={:08X}{}",
            j, pc, instr, mode_str, if thumb {"T"} else {"A"}, emu.cpu.r[14], emu.cpu.r[0], swi);
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.r[15] == 0x08000190 {
            eprintln!("[225100+{}] *** RESTARTED (EWRAM clear) ***", j+1);
            break;
        }
        if emu.cpu.r[15] == 0 || emu.cpu.r[15] == 0x08000000 {
            eprintln!("[225100+{}] *** JUMP to 0x{:08X} ***", j+1, emu.cpu.r[15]);
        }
    }
}
