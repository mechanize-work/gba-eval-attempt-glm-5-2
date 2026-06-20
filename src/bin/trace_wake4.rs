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
    
    // Run 1 frame to reach VBlankIntrWait
    gba_emu::emulator::run_frame();
    eprintln!("Frame 0 done: halted={} vbiw={}", emu.cpu.halted, emu.cpu.vblank_intr_wait);
    
    // Step through frame 1 manually, tracking VBlank wake-up
    let target = emu.cycle_count.wrapping_add(280896);
    let mut count = 0u64;
    let mut woke = false;
    let mut last_scan = emu.current_scanline;
    
    while emu.cycle_count < target && count < 500000 {
        gba_emu::emulator::step_one();
        count += 1;
        
        if emu.current_scanline != last_scan {
            if emu.current_scanline == 160 {
                eprintln!("[{}] VBlank at scanline 160! vb_occ={} halted={} vbiw={}",
                    count, emu.vblank_occurred, emu.cpu.halted, emu.cpu.vblank_intr_wait);
            }
            last_scan = emu.current_scanline;
        }
        
        if !emu.cpu.halted && !woke && count > 1000 {
            eprintln!("[{}] CPU woke up! PC=0x{:08X} vbiw={} [15E0]=0x{:08X}",
                count, emu.cpu.r[15], emu.cpu.vblank_intr_wait, emu.mem.read_word(0x030015E0));
            woke = true;
            break;
        }
    }
    
    if !woke {
        eprintln!("CPU never woke up after {} steps. halted={} vbiw={} vb_occ={}",
            count, emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred);
    }
}
