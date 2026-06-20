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
    
    // Run 2 frames using run_frame
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    eprintln!("Before rf3: halted={} vbiw={} vb_occ={} cc={} scan={} cycle_in_scan={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.cycle_count, emu.current_scanline, emu.cycle_in_scanline);
    
    // Now manually run rf3's exact code, with scanline tracking
    let target = emu.cycle_count.wrapping_add(280896);
    let mut instr_count: u64 = 0;
    let mut last_scan = emu.current_scanline;
    let mut vblank_count = 0;
    
    while emu.cycle_count < target && instr_count < 2_000_000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
            instr_count += 1;
            
            if emu.current_scanline != last_scan {
                if emu.current_scanline == 160 {
                    vblank_count += 1;
                    eprintln!("[{}] VBlank #{}! scan=160 vb_occ={} halted={} vbiw={}",
                        instr_count, vblank_count, emu.vblank_occurred,
                        emu.cpu.halted, emu.cpu.vblank_intr_wait);
                }
                last_scan = emu.current_scanline;
            }
        } else {
            emu.execute_one();
            instr_count += 1;
            emu.check_and_handle_interrupts();
        }
    }
    
    eprintln!("After manual rf3: halted={} [15E0]={} cc={} count={} vblank_count={}",
        emu.cpu.halted, emu.mem.read_word(0x030015E0), emu.cycle_count, instr_count, vblank_count);
    
    // Do wrapping_sub + render (same as run_frame end)
    emu.cycle_count = emu.cycle_count.wrapping_sub(280896);
    emu.ppu.render_frame(&emu.mem);
    
    // Now run rf4 using run_frame
    gba_emu::emulator::run_frame();
    eprintln!("After rf4 (run_frame): halted={} [15E0]={}",
        emu.cpu.halted, emu.mem.read_word(0x030015E0));
}
