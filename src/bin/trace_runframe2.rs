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
    
    // Run 1 frame (init)
    gba_emu::emulator::run_frame();
    eprintln!("After frame 0: halted={} PC=0x{:08X} cc={} scan={}",
        emu.cpu.halted, emu.cpu.r[15], emu.cycle_count, emu.current_scanline);
    
    // Run frame 1 (should reach VBlankIntrWait)
    gba_emu::emulator::run_frame();
    eprintln!("After frame 1: halted={} vbiw={} PC=0x{:08X} cc={} scan={} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cpu.r[15], emu.cycle_count, emu.current_scanline,
        emu.mem.read_word(0x030015E0));
    
    // Run frame 2 - this should wake from VBlankIntrWait and run init
    // But add detailed tracking
    let start_cc = emu.cycle_count;
    let target = start_cc.wrapping_add(280896);
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    let mut last_scan = emu.current_scanline;
    let mut last_halted = emu.cpu.halted;
    let mut instr_count = 0u64;
    
    while emu.cycle_count < target && instr_count < 2_000_000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
        } else {
            emu.execute_one();
            emu.check_and_handle_interrupts();
        }
        instr_count += 1;
        
        if emu.current_scanline != last_scan {
            if emu.current_scanline == 160 {
                eprintln!("[{}] VBlank! halted={} vbiw={} vb_occ={} [15E0]=0x{:08X}",
                    instr_count, emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
                    emu.mem.read_word(0x030015E0));
            }
            last_scan = emu.current_scanline;
        }
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X} halted={}",
                instr_count, last_15e0, v15e0, emu.cpu.r[15], emu.cpu.halted);
            last_15e0 = v15e0;
        }
        
        if emu.cpu.halted != last_halted {
            eprintln!("[{}] halted changed: {} -> {} PC=0x{:08X}",
                instr_count, last_halted, emu.cpu.halted, emu.cpu.r[15]);
            last_halted = emu.cpu.halted;
        }
    }
    
    eprintln!("Frame 2 done: halted={} PC=0x{:08X} [15E0]=0x{:08X} cc={} count={}",
        emu.cpu.halted, emu.cpu.r[15], last_15e0, emu.cycle_count, instr_count);
}
