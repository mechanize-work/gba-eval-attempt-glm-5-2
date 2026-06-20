// Debug cycle counting
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
    
    eprintln!("After load_rom: cycle_count={} cpu.cycles={} PC=0x{:08X}", 
        emu.cycle_count, emu.cpu.cycles, emu.cpu.r[15]);
    
    // Execute 10 instructions manually and check cycle accumulation
    for i in 0..10 {
        let before_cc = emu.cycle_count;
        let before_cpu_c = emu.cpu.cycles;
        gba_emu::emulator::step_one();
        eprintln!("[{}] PC=0x{:08X} cycle_count {}->{} cpu.cycles {}->{}",
            i, emu.cpu.r[15], before_cc, emu.cycle_count, before_cpu_c, emu.cpu.cycles);
    }
    
    // Now run a frame and check
    let before = emu.cycle_count;
    gba_emu::emulator::run_frame();
    eprintln!("\nrun_frame: cycle_count {}->{} (delta={})", before, emu.cycle_count, emu.cycle_count.wrapping_sub(before));
    eprintln!("PC=0x{:08X}", emu.cpu.r[15]);
}
