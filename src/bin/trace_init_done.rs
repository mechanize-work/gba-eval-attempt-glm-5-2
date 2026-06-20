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
    
    // Step through 500K instructions, logging when PC enters new ranges
    let mut last_range = 0x08000100u32;
    for i in 0..500_000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        let range = pc & 0xFFFF_FE00; // 512-byte granularity
        
        if range != last_range {
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            let halted = emu.cpu.halted;
            let sp = emu.cpu.r[13];
            let lr = emu.cpu.r[14];
            eprintln!("[{}] PC=0x{:08X} DC=0x{:04X} h={} SP=0x{:08X} LR=0x{:08X} R0={:08X} R1={:08X}",
                i, pc, dispcnt, halted, sp, lr, emu.cpu.r[0], emu.cpu.r[1]);
            last_range = range;
        }
        
        if pc == 0 || emu.cpu.halted {
            eprintln!("[{}] STOPPED: PC=0x{:08X} halted={}", i, pc, emu.cpu.halted);
            break;
        }
    }
}
