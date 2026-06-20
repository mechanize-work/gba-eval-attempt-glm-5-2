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
    
    // Call run_frame manually but with debug prints
    let target = emu.cycle_count + 280896;
    eprintln!("cycle_count={}, target={}", emu.cycle_count, target);
    
    let mut count = 0u32;
    while emu.cycle_count < target && count < 10 {
        let before_cc = emu.cycle_count;
        let before_cpu_cyc = emu.cpu.cycles;
        gba_emu::emulator::step_one();
        eprintln!("[{}] PC=0x{:08X} cc {}->{} cpu.cyc {}->{}",
            count, emu.cpu.r[15], before_cc, emu.cycle_count, before_cpu_cyc, emu.cpu.cycles);
        count += 1;
    }
    eprintln!("After {} steps, cycle_count={}, target={}", count, emu.cycle_count, target);
}
