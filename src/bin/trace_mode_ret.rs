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
    
    for i in 0..225130 { gba_emu::emulator::step_one(); }
    
    for j in 0..60u32 {
        let pc = emu.cpu.r[15];
        let mode = emu.cpu.cpsr & 0x1F;
        let sp = emu.cpu.r[13];
        let lr = emu.cpu.r[14];
        let usr_sp = emu.cpu.usr_r13;
        let irq_sp = emu.cpu.irq_r13;
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        let mode_str = match mode { 0x10=>"USR",0x11=>"FIQ",0x12=>"IRQ",0x13=>"SVC",0x1F=>"SYS",_=>&"??" };
        
        eprintln!("[{}] PC=0x{:08X} 0x{:X} {}{} SP=0x{:08X} LR=0x{:08X} usr_sp=0x{:08X} irq_sp=0x{:08X}",
            j, pc, instr, mode_str, if thumb {"T"} else {"A"}, sp, lr, usr_sp, irq_sp);
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.r[15] == 0x08000190 && j > 5 { eprintln!("RESTART!"); break; }
        if emu.cpu.r[15] == 0 { eprintln!("JUMP 0!"); }
    }
}
