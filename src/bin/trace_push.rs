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
    
    // Run 1 frame (should complete EWRAM clear + BSS clear + start game init)
    gba_emu::emulator::run_frame();
    
    // Now trace step by step
    for i in 0..200u64 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let sp = emu.cpu.r[13];
        let lr = emu.cpu.r[14];
        
        // Decode the instruction
        let decode = if thumb {
            let bits15_10 = (instr >> 10) & 0x3F;
            match bits15_10 {
                0x2C => "ADD_SP",
                0x2D => "PUSH",
                0x2F => "POP",
                0x30..=0x33 => "STM/LDM",
                0x11 => {
                    let op = (instr >> 8) & 3;
                    match op { 3 => "BX", _ => "HI_REG" }
                }
                _ => ""
            }
        } else { "" };
        
        eprintln!("[{}] PC=0x{:08X} 0x{:04X} {} SP=0x{:08X} LR=0x{:08X} R0={:08X} R1={:08X}",
            i, pc, instr, decode, sp, lr, emu.cpu.r[0], emu.cpu.r[1]);
        
        gba_emu::emulator::step_one();
        
        if pc == 0 || emu.cpu.halted {
            eprintln!("STOPPED: PC=0x{:08X} halted={}", emu.cpu.r[15], emu.cpu.halted);
            break;
        }
    }
}
