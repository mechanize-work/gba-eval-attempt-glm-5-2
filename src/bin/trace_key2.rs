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
    
    // Run to VBlankIntrWait
    for i in 0..226834 { gba_emu::emulator::step_one(); }
    
    // Run 2 frames to get to key check
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    // Check KEYINPUT
    let ki = emu.mem.read_half(0x04000130);
    eprintln!("KEYINPUT = 0x{:04X}", ki);
    eprintln!("io[0x130]=0x{:02X} io[0x131]=0x{:02X}", emu.mem.io[0x130], emu.mem.io[0x131]);
    
    // Now check what the game reads
    let pc = emu.cpu.r[15];
    let instr = emu.mem.read_half(pc) as u32;
    eprintln!("PC=0x{:08X} instr=0x{:04X}", pc, instr);
    
    // Step and check
    for j in 0..10 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        gba_emu::emulator::step_one();
        
        // Check if this instruction reads from 0x04000130
        if (instr & 0xF800) == 0x8800 { // LDRB
            let rn = (instr >> 3) & 7;
            let off = (instr >> 6) & 0x1F;
            let addr = emu.cpu.r[rn as usize].wrapping_add(off);
            // This is the PRE-step value, but we already stepped
            // Actually the address is computed from PRE-step registers
            // But rn and off are from the instruction
        }
        
        // Show KEYINPUT value
        let ki = (emu.mem.io[0x130] as u16) | ((emu.mem.io[0x131] as u16) << 8);
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} KEYINPUT=0x{:04X} R2={:08X} R3={:08X} R4={:08X}",
            j, pc, instr, ki, emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[4]);
    }
}
