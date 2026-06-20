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
    
    // Track [0x030015E0] changes
    let mut last_15e0 = 0u32;
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        let v = emu.mem.read_word(0x030015E0);
        if v != last_15e0 {
            eprintln!("[{}] [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X} R0={:08X} R1={:08X}",
                i, last_15e0, v, emu.cpu.r[15], emu.cpu.r[0], emu.cpu.r[1]);
            last_15e0 = v;
        }
        
        // Also track when PC reaches key addresses
        let pc = emu.cpu.r[15];
        if pc == 0x0800030C || pc == 0x08000316 || pc == 0x0800032A || pc == 0x08013468 || pc == 0x08013420 {
            eprintln!("[{}] PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                i, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted at PC=0x{:08X}", i, emu.cpu.r[15]);
            break;
        }
    }
}
