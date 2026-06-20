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
    
    for _ in 0..3 { gba_emu::emulator::run_frame(); }
    
    // State after 3 rf
    eprintln!("After 3 rf: halted={} vbiw={} vb_occ={} cc={} scan={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.cycle_count, emu.current_scanline);
    
    // Manually replicate run_frame EXACTLY (copy-paste from source)
    let target_cycles = emu.cycle_count.wrapping_add(280896);
    let mut instr_count: u64 = 0;
    
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
    }
    
    eprintln!("After manual loop: halted={} [15E0]={} cc={} count={}",
        emu.cpu.halted, emu.mem.read_word(0x030015E0), emu.cycle_count, instr_count);
    
    // Now do what run_frame does after the loop
    emu.cycle_count = emu.cycle_count.wrapping_sub(280896);
    emu.ppu.render_frame(&emu.mem);
    
    eprintln!("After sub+render: halted={} [15E0]={} cc={}",
        emu.cpu.halted, emu.mem.read_word(0x030015E0), emu.cycle_count);
    
    // Now call run_frame for the NEXT frame
    emu.run_frame();
    eprintln!("After next run_frame: halted={} [15E0]={}",
        emu.cpu.halted, emu.mem.read_word(0x030015E0));
}
