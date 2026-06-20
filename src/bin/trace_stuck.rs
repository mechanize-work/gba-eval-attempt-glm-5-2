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
    
    // Run 5 frames to get past init
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Trace 50 instructions from current position
    for j in 0..50 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let nzcv = format!("{}{}{}{}",
            if emu.cpu.cpsr & 0x80000000 != 0 { 'N' } else { '-' },
            if emu.cpu.cpsr & 0x40000000 != 0 { 'Z' } else { '-' },
            if emu.cpu.cpsr & 0x20000000 != 0 { 'C' } else { '-' },
            if emu.cpu.cpsr & 0x10000000 != 0 { 'V' } else { '-' },
        );
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} SP={:08X} LR={:08X} {}",
            j, pc, instr,
            emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
            emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7],
            emu.cpu.r[13], emu.cpu.r[14], nzcv);
        gba_emu::emulator::step_one();
        
        if emu.cpu.halted {
            eprintln!("[{}] HALTED", j+1);
            let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
            let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
            let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
            eprintln!("  IME={} IE=0x{:04X} IF=0x{:04X}", ime, ie, if_);
            break;
        }
    }
}
