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
    
    // Step through many instructions, logging when PC changes to new ranges
    let mut last_range = 0u32;
    for i in 0..5_000_000u32 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        let range = pc & 0xFFFF_F000; // 4K granularity
        
        if range != last_range {
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            let halted = emu.cpu.halted;
            eprintln!("[{}] PC=0x{:08X} (new range 0x{:08X}) DISPCNT=0x{:04X} halted={} R0=0x{:08X} R1=0x{:08X}",
                i, pc, range, dispcnt, halted, emu.cpu.r[0], emu.cpu.r[1]);
            last_range = range;
        }
        
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dispcnt != 0 {
            eprintln!("[{}] DISPCNT=0x{:04X} set! PC=0x{:08X}", i, dispcnt, pc);
            break;
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU halted! PC=0x{:08X}", i, pc);
            // Check if it would wake up
            let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
            let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
            let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
            eprintln!("  IME={} IE=0x{:04X} IF=0x{:04X}", ime, ie, if_);
            break;
        }
    }
}
