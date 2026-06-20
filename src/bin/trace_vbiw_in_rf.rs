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
    
    // Run 1 frame
    gba_emu::emulator::run_frame();
    
    // Run frame 1 step by step, counting how many times check_and_handle_interrupts
    // sees halted=true, vbiw=true, vb_occ=true
    let target = emu.cycle_count.wrapping_add(280896);
    let mut count = 0u64;
    let mut vbiw_hits = 0u32;
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    
    while emu.cycle_count < target && count < 500000 {
        // Manually check the VBlankIntrWait condition BEFORE calling check_and_handle_interrupts
        let will_hit_vbiw = emu.cpu.halted && emu.cpu.vblank_intr_wait && emu.vblank_occurred;
        
        emu.check_and_handle_interrupts();
        
        if will_hit_vbiw {
            vbiw_hits += 1;
            let v15e0 = emu.mem.read_word(0x030015E0);
            eprintln!("[{}] VBlankIntrWait would fire! [15E0]=0x{:08X}->0x{:08X} halted={}",
                count, last_15e0, v15e0, emu.cpu.halted);
            last_15e0 = v15e0;
        }
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
        } else {
            emu.execute_one();
            emu.check_and_handle_interrupts();
        }
        count += 1;
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] changed to 0x{:08X} at PC=0x{:08X}", count, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
        }
    }
    
    eprintln!("Frame 1 done: vbiw_hits={} [15E0]=0x{:08X} count={}", vbiw_hits, last_15e0, count);
}
