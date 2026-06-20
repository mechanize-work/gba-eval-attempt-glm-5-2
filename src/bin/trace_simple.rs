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
    eprintln!("After 2 frames: PC=0x{:08X} halted={}", emu.cpu.r[15], emu.cpu.halted);
    
    // Step until not halted, then trace
    let mut steps = 0u64;
    while emu.cpu.halted && steps < 300000 {
        gba_emu::emulator::step_one();
        steps += 1;
    }
    eprintln!("Woke up after {} steps. PC=0x{:08X}", steps, emu.cpu.r[15]);
    
    // Trace 30 instructions
    for j in 0..30 {
        let pc = emu.cpu.r[15];
        let instr = if emu.cpu.is_thumb() { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let v15e0 = emu.mem.read_word(0x030015E0);
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        eprintln!("[{}] PC=0x{:08X} 0x{:X} [15E0]=0x{:08X} DC=0x{:04X} R0={:08X} R1={:08X} LR={:08X}",
            j, pc, instr, v15e0, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[14]);
        gba_emu::emulator::step_one();
        if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
    }
}
