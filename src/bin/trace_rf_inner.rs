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
    
    // Run 2 frames
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    eprintln!("State: halted={} vbiw={} vb_occ={} cc={} scan={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.cycle_count, emu.current_scanline);
    
    // Manually run the EXACT run_frame loop with debug
    let target = emu.cycle_count.wrapping_add(280896);
    let mut instr_count: u64 = 0;
    let mut vbiw_fired = false;
    
    while emu.cycle_count < target && instr_count < 2_000_000 {
        // Check BEFORE
        let halted_before = emu.cpu.halted;
        let vbiw_before = emu.cpu.vblank_intr_wait;
        let vb_occ_before = emu.vblank_occurred;
        
        emu.check_and_handle_interrupts();
        
        // Check if VBlankIntrWait fired
        if halted_before && vbiw_before && vb_occ_before && !emu.cpu.halted {
            vbiw_fired = true;
            eprintln!("[{}] VBlankIntrWait fired! [15E0]=0x{:08X} PC=0x{:08X}",
                instr_count, emu.mem.read_word(0x030015E0), emu.cpu.r[15]);
        }
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
            instr_count += 1;
        } else {
            emu.execute_one();
            instr_count += 1;
            emu.check_and_handle_interrupts();
        }
        
        if vbiw_fired && instr_count % 10000 == 0 {
            eprintln!("[{}] PC=0x{:08X} halted={} [15E0]=0x{:08X} cc={}",
                instr_count, emu.cpu.r[15], emu.cpu.halted,
                emu.mem.read_word(0x030015E0), emu.cycle_count);
        }
    }
    
    eprintln!("Result: vbiw_fired={} halted={} PC=0x{:08X} [15E0]=0x{:08X} count={}",
        vbiw_fired, emu.cpu.halted, emu.cpu.r[15],
        emu.mem.read_word(0x030015E0), instr_count);
}
