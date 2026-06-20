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
    
    eprintln!("After 2 rf: halted={} vbiw={} vb_occ={} [15E0]={} cc={} scan={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0), emu.cycle_count, emu.current_scanline);
    
    // Now manually run EXACTLY what run_frame does, with debug
    let target_cycles = emu.cycle_count.wrapping_add(280896);
    let mut instr_count: u64 = 0;
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    
    while emu.cycle_count < target_cycles && instr_count < 2_000_000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
            instr_count += 1;
        } else {
            emu.execute_one();
            instr_count += 1;
            emu.check_and_handle_interrupts();
        }
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X} halted={} cc={}",
                instr_count, last_15e0, v15e0, emu.cpu.r[15], emu.cpu.halted, emu.cycle_count);
            last_15e0 = v15e0;
        }
    }
    
    eprintln!("Manual: halted={} PC=0x{:08X} [15E0]=0x{:08X} cc={} count={}",
        emu.cpu.halted, emu.cpu.r[15], last_15e0, emu.cycle_count, instr_count);
    
    // Do wrapping_sub and render (same as run_frame)
    emu.cycle_count = emu.cycle_count.wrapping_sub(280896);
    emu.ppu.render_frame(&emu.mem);
    
    // Now run_frame for frame 4
    gba_emu::emulator::run_frame();
    eprintln!("After rf4: halted={} PC=0x{:08X} [15E0]=0x{:08X}",
        emu.cpu.halted, emu.cpu.r[15], emu.mem.read_word(0x030015E0));
}
