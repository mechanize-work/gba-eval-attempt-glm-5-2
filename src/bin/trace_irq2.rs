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
    
    // Run 2 frames to get past init
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    eprintln!("After 2 frames: PC=0x{:08X} halted={}", emu.cpu.r[15], emu.cpu.halted);
    eprintln!("[0x03007FFC] = 0x{:08X}", emu.mem.read_word(0x03007FFC));
    
    // Trace 100 instructions
    for j in 0..100 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let v15e0 = emu.mem.read_word(0x030015E0);
        let halted = emu.cpu.halted;
        
        // Show when in IWRAM or BIOS
        if pc < 0x04000000 || dc != 0x0080 || v15e0 != 0 || j < 10 {
            eprintln!("[{:2}] PC=0x{:08X} 0x{:X} DC=0x{:04X} [15E0]=0x{:08X} h={} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}",
                j, pc, instr, dc, v15e0, halted,
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[13], emu.cpu.r[14]);
        }
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
    }
}
