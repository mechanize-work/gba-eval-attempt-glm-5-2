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
    
    // Step until we reach 0x08000130 (after first memset)
    for i in 0..500_000u32 {
        gba_emu::emulator::step_one();
        if emu.cpu.r[15] == 0x08000130 {
            eprintln!("At 0x08000130, step {}", i);
            
            // Now trace the LDR instructions
            for j in 0..15 {
                let pc = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc) as u32;
                
                // Before execution
                let r0_before = emu.cpu.r[0];
                let r1_before = emu.cpu.r[1];
                let r2_before = emu.cpu.r[2];
                
                gba_emu::emulator::step_one();
                
                eprintln!("[{}] PC=0x{:08X} 0x{:04X} R0: 0x{:08X}->0x{:08X} R1: 0x{:08X}->0x{:08X} R2: 0x{:08X}->0x{:08X}",
                    j, pc, instr, r0_before, emu.cpu.r[0], r1_before, emu.cpu.r[1], r2_before, emu.cpu.r[2]);
            }
            break;
        }
    }
}
