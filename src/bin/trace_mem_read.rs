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
    
    // Step to just before 282188 (where R6 gets set to 0x0300243C)
    for i in 0..282185 {
        gba_emu::emulator::step_one();
    }
    
    // Trace 15 instructions around the R6 set and the LDR
    for j in 0..15 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let r2 = emu.cpu.r[2];
        let r6 = emu.cpu.r[6];
        
        eprintln!("[282185+{}] PC=0x{:08X} 0x{:04X} R2={:08X} R6={:08X} R0={:08X} R1={:08X} R3={:08X} R4={:08X} R5={:08X} R7={:08X} SP={:08X} LR={:08X}",
            j, pc, instr, r2, r6, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[3], emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[7], emu.cpu.r[13], emu.cpu.r[14]);
        
        // If this is a LDR from [R6], show what memory contains
        if (instr & 0xF800) == 0x6800 { // LDR Rd, [Rn, #imm]
            let rn = (instr >> 3) & 7;
            if rn == 6 {
                let addr = r6;
                let val = emu.mem.read_word(addr);
                eprintln!("  -> LDR from [R6]=[0x{:08X}] = 0x{:08X}", addr, val);
            }
        }
        
        gba_emu::emulator::step_one();
    }
}
