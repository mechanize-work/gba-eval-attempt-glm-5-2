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
    
    // Run to VBlankIntrWait (step 226834)
    for i in 0..226834 { gba_emu::emulator::step_one(); }
    
    // Now run 2 more frames (should wake from VBIW and loop)
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    // Now trace the key check loop in detail
    for j in 0..30 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        
        // Decode key-related instructions
        let decode = if (instr & 0xF800) == 0x8800 { // LDRB Rd, [Rn, #imm]
            let rn = (instr >> 3) & 7;
            let rd = instr & 7;
            let off = (instr >> 6) & 0x1F;
            let addr = emu.cpu.r[rn as usize].wrapping_add(off);
            let val = emu.mem.read_byte(addr);
            format!("LDRB R{} = [R{}+#{:02X}] = [0x{:08X}] = 0x{:02X}", rd, rn, off, addr, val)
        } else if (instr & 0xFF00) == 0x4200 { // TST/CMP
            let rd = instr & 7;
            format!("TST R{}, R{}", (instr>>3)&7, rd)
        } else {
            String::new()
        };
        
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} {} R2={:08X} R3={:08X} R4={:08X} R5={:08X}",
            j, pc, instr, decode, emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[4], emu.cpu.r[5]);
        
        gba_emu::emulator::step_one();
    }
}
