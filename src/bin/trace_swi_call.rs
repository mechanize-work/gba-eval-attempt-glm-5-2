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
    
    // Run 5 frames to get to the SWI call
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Now step and find the SWI call
    for i in 0..200 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        
        // Check if this is SWI 0x05
        if (instr & 0xFF00) == 0xDF00 {
            eprintln!("[{}] SWI {} at PC=0x{:08X}", i, instr & 0xFF, pc);
            eprintln!("  Before: halted={} R0={:08X} R1={:08X} SP={:08X} LR={:08X}",
                emu.cpu.halted, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[13], emu.cpu.r[14]);
        }
        
        gba_emu::emulator::step_one();
        
        if (instr & 0xFF00) == 0xDF00 {
            eprintln!("  After:  halted={} PC=0x{:08X} R0={:08X} R1={:08X} SP={:08X} LR={:08X}",
                emu.cpu.halted, emu.cpu.r[15], emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[13], emu.cpu.r[14]);
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i+1, emu.cpu.r[15]);
            // Check what happens next - does it wake up?
            for j in 0..10 {
                let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
                let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
                let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
                eprintln!("  [{}+{}] halted={} IME={} IE=0x{:04X} IF=0x{:04X} PC=0x{:08X}",
                    i+1, j, emu.cpu.halted, ime, ie, if_, emu.cpu.r[15]);
                gba_emu::emulator::step_one();
                if !emu.cpu.halted {
                    eprintln!("  CPU woke up!");
                    break;
                }
            }
            break;
        }
    }
}
