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
    
    // Run 1 frame to get to halt
    gba_emu::emulator::run_frame();
    
    eprintln!("After frame 0: halted={} scanline={} vb_occ={} cycle_count={} cycle_in_scanline={}",
        emu.cpu.halted, emu.current_scanline, emu.vblank_occurred, emu.cycle_count, emu.cycle_in_scanline);
    
    // Now manually run the run_frame loop for frame 1
    let target = emu.cycle_count.wrapping_add(280896);
    let mut count = 0u64;
    let mut last_scan = emu.current_scanline;
    
    while emu.cycle_count < target && count < 500000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
            
            if emu.current_scanline != last_scan {
                if emu.current_scanline == 160 {
                    eprintln!("[{}] VBlank at scanline 160! vb_occ={} halted={}",
                        count, emu.vblank_occurred, emu.cpu.halted);
                }
                last_scan = emu.current_scanline;
            }
            
            if !emu.cpu.halted {
                eprintln!("[{}] CPU woke up! PC=0x{:08X} scanline={}",
                    count, emu.cpu.r[15], emu.current_scanline);
            }
        } else {
            emu.execute_one();
        }
        count += 1;
    }
    
    eprintln!("After frame 1: halted={} scanline={} count={} cycle_count={}",
        emu.cpu.halted, emu.current_scanline, count, emu.cycle_count);
}
