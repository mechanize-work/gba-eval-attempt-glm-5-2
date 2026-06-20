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
    
    // Run frame 0 manually and see what happens during run_frame
    // The key question: does the actual run_frame method work?
    // Let me call the global run_frame and check state before/after
    
    let before_cc = emu.cycle_count;
    let before_pc = emu.cpu.r[15];
    eprintln!("Before run_frame: cc={}, PC=0x{:08X}", before_cc, before_pc);
    
    gba_emu::emulator::run_frame();
    
    eprintln!("After run_frame: cc={}, PC=0x{:08X}", emu.cycle_count, emu.cpu.r[15]);
    eprintln!("  Delta cc={}", emu.cycle_count.wrapping_sub(before_cc));
    
    // Now call it again
    let before2 = emu.cycle_count;
    gba_emu::emulator::run_frame();
    eprintln!("After run_frame 2: cc={}, PC=0x{:08X}, delta={}", 
        emu.cycle_count, emu.cpu.r[15], emu.cycle_count.wrapping_sub(before2));
    
    // And again
    let before3 = emu.cycle_count;
    gba_emu::emulator::run_frame();
    eprintln!("After run_frame 3: cc={}, PC=0x{:08X}, delta={}", 
        emu.cycle_count, emu.cpu.r[15], emu.cycle_count.wrapping_sub(before3));
}
