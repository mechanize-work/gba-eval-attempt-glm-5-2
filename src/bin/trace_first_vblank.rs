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
    
    // Step to first VBlankIntrWait (step 226834)
    for i in 0..226834 { gba_emu::emulator::step_one(); }
    
    // Execute the VBlankIntrWait (should halt and wake on VBlank)
    gba_emu::emulator::step_one(); // SWI 0x05
    
    // Now the CPU should be halted. Run until it wakes up
    let mut wake_step = 0;
    for j in 0..300_000 {
        gba_emu::emulator::step_one();
        if !emu.cpu.halted {
            wake_step = j;
            break;
        }
    }
    eprintln!("CPU woke up after {} steps", wake_step);
    
    // Now trace 50 instructions after waking up
    for k in 0..50 {
        let pc = emu.cpu.r[15];
        let instr = emu.mem.read_half(pc) as u32;
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let is_swi = (instr & 0xFF00) == 0xDF00;
        
        let swi_info = if is_swi { format!(" *** SWI {} ***", instr & 0xFF) } else { String::new() };
        
        eprintln!("[{:3}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}{}",
            k, pc, instr, dc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], swi_info);
        
        gba_emu::emulator::step_one();
        
        let dc2 = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc2 != dc {
            eprintln!("  *** DISPCNT changed: 0x{:04X} -> 0x{:04X} ***", dc, dc2);
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] HALTED", k+1);
            break;
        }
    }
}
