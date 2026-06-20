// Debug: count instructions per frame
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
    
    // Run frames manually using step_one and count
    for frame in 0..5 {
        let mut instr_count = 0u32;
        let start_cycles = emu.cycle_count;
        let target = start_cycles + 280896;
        
        while emu.cycle_count < target && instr_count < 5_000_000 {
            gba_emu::emulator::step_one();
            instr_count += 1;
        }
        
        eprintln!("Frame {}: {} instructions, cycles {}->{} (target {}), PC=0x{:08X}",
            frame, instr_count, start_cycles, emu.cycle_count, target, emu.cpu.r[15]);
    }
}
