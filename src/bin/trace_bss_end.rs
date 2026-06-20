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
    
    // Run frames until we see PC leave 0x08000190-0x08000196
    for frame in 0..700 {
        gba_emu::emulator::run_frame();
        let pc = emu.cpu.r[15];
        let r1 = emu.cpu.r[1];
        
        if pc < 0x08000190 || pc > 0x08000196 {
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
            let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
            let halted = emu.cpu.halted;
            eprintln!("Frame {}: EXITED BSS clear! PC=0x{:08X} R1=0x{:08X} DISPCNT=0x{:04X} IME={} IE=0x{:04X} halted={}",
                frame, pc, r1, dispcnt, ime, ie, halted);
            
            // Trace 50 instructions
            for j in 0..50 {
                let pc2 = emu.cpu.r[15];
                let thumb = emu.cpu.is_thumb();
                let instr: u32 = if thumb { emu.mem.read_half(pc2) as u32 } else { emu.mem.read_word(pc2) };
                let r0 = emu.cpu.r[0];
                let r1 = emu.cpu.r[1];
                let r2 = emu.cpu.r[2];
                let r3 = emu.cpu.r[3];
                let sp = emu.cpu.r[13];
                let lr = emu.cpu.r[14];
                let halted = emu.cpu.halted;
                
                // Check for SWI
                if thumb && (instr & 0xFF00) == 0xDF00 {
                    eprintln!("  [{}] SWI {} at PC=0x{:08X}", j, instr & 0xFF, pc2);
                }
                
                eprintln!("  [{}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X} halted={}",
                    j, pc2, instr, r0, r1, r2, r3, sp, lr, halted);
                
                gba_emu::emulator::step_one();
                
                if emu.cpu.halted && j > 0 {
                    eprintln!("  [{}] CPU halted!", j+1);
                    break;
                }
            }
            break;
        }
    }
}
