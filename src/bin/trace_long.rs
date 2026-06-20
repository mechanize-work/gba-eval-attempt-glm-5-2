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
    
    // Run 500 frames (should be enough to get past any init)
    for frame in 0..500 {
        gba_emu::emulator::run_frame();
        
        let pc = emu.cpu.r[15];
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        
        if dispcnt != 0 || (pc < 0x08000100) || (pc > 0x08000200) {
            eprintln!("Frame {}: PC=0x{:08X} DISPCNT=0x{:04X} R0=0x{:08X} R1=0x{:08X}",
                frame, pc, dispcnt, emu.cpu.r[0], emu.cpu.r[1]);
        }
        
        if dispcnt != 0 {
            eprintln!("DISPCNT set! Breaking.");
            break;
        }
        
        if emu.cpu.halted {
            eprintln!("CPU halted at PC=0x{:08X}", pc);
            break;
        }
    }
}
