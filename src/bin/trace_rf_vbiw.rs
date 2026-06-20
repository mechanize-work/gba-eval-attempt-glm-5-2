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
    
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    eprintln!("After 2 frames: halted={} vbiw={} vb_occ={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred);
    
    // Manually replicate run_frame's loop
    let target = emu.cycle_count.wrapping_add(280896);
    let mut count = 0u64;
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    
    while emu.cycle_count < target && count < 2_000_000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
            count += 1;
        } else {
            emu.execute_one();
            count += 1;
            emu.check_and_handle_interrupts();
        }
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X} halted={}",
                count, last_15e0, v15e0, emu.cpu.r[15], emu.cpu.halted);
            last_15e0 = v15e0;
        }
    }
    
    eprintln!("After manual frame: halted={} PC=0x{:08X} [15E0]=0x{:08X} count={}",
        emu.cpu.halted, emu.cpu.r[15], last_15e0, count);
}
