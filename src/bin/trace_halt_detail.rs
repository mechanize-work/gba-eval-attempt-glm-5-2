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
    
    // Run 2 frames to reach halt
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    eprintln!("Halted: {} vbiw: {} vb_occ: {}", emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred);
    eprintln!("IE: 0x{:04X} IF: 0x{:04X} IME: {}", 
        (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8),
        (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8),
        (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8));
    eprintln!("CPSR: 0x{:08X} I_flag: {}", emu.cpu.cpsr, emu.cpu.cpsr & 0x80 != 0);
    
    // Step through halt loop, tracking everything
    let mut last_scan = emu.current_scanline;
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        if emu.current_scanline != last_scan {
            if emu.current_scanline == 160 {
                eprintln!("[{}] VBlank! scanline=160 vb_occ={} halted={} vbiw={} [15E0]=0x{:08X} IF=0x{:04X}",
                    i, emu.vblank_occurred, emu.cpu.halted, emu.cpu.vblank_intr_wait,
                    emu.mem.read_word(0x030015E0),
                    (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8));
            }
            last_scan = emu.current_scanline;
        }
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] changed to 0x{:08X} at PC=0x{:08X}", i, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
        }
        
        if !emu.cpu.halted {
            eprintln!("[{}] CPU woke up! PC=0x{:08X} [15E0]=0x{:08X}", i, emu.cpu.r[15], v15e0);
            break;
        }
        
        if i % 100000 == 0 && i > 0 {
            eprintln!("[{}] Still halted. scanline={} vb_occ={}", i, emu.current_scanline, emu.vblank_occurred);
        }
    }
}
