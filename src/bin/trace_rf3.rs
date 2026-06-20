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
    
    // Call the actual run_frame and see what happens
    let before_cc = emu.cycle_count;
    let before_pc = emu.cpu.r[15];
    
    gba_emu::emulator::run_frame();
    
    eprintln!("Before: cc={}, PC=0x{:08X}", before_cc, before_pc);
    eprintln!("After:  cc={}, PC=0x{:08X}", emu.cycle_count, emu.cpu.r[15]);
    eprintln!("Delta:  {}", emu.cycle_count.wrapping_sub(before_cc));
    
    // Now manually run the same number of steps
    gba_emu::emulator::reset();
    // Reload ROM
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    let before2_cc = emu.cycle_count;
    let target = before2_cc.wrapping_add(280896u32);
    let mut count = 0u64;
    while emu.cycle_count < target && count < 2_000_000 {
        gba_emu::emulator::step_one();
        count += 1;
    }
    eprintln!("\nManual: cc {}->{}, {} instructions, PC=0x{:08X}", 
        before2_cc, emu.cycle_count, count, emu.cpu.r[15]);
}
