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
    
    // Trace all instructions from start until we reach the first memset loop
    for i in 0..200u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        // Check for SWI
        if thumb && (instr & 0xFF00) == 0xDF00 {
            eprintln!("[{}] SWI {} at PC=0x{:08X}", i, instr & 0xFF, pc);
        } else if !thumb && (instr >> 24) == 0xEF {
            eprintln!("[{}] SWI 0x{:06X} at PC=0x{:08X}", i, instr & 0xFFFFFF, pc);
        }
        
        let nzcv = format!("{}{}{}{}",
            if emu.cpu.cpsr & 0x80000000 != 0 { 'N' } else { '-' },
            if emu.cpu.cpsr & 0x40000000 != 0 { 'Z' } else { '-' },
            if emu.cpu.cpsr & 0x20000000 != 0 { 'C' } else { '-' },
            if emu.cpu.cpsr & 0x10000000 != 0 { 'V' } else { '-' },
        );
        
        eprintln!("[{:3}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} SP={:08X} LR={:08X} {}",
            i, pc, instr,
            emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
            emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7],
            emu.cpu.r[13], emu.cpu.r[14], nzcv);
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.r[15] == 0x08000190 {
            eprintln!("[{}] Reached memset loop!", i + 1);
            break;
        }
    }
}
