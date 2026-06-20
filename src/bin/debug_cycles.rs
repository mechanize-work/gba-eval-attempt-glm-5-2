// Debug: check cycle execution per frame
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
    
    for frame in 0..10 {
        let before_cycles = emu.cycle_count;
        let before_pc = emu.cpu.r[15];
        let before_r0 = emu.cpu.r[0];
        let before_r1 = emu.cpu.r[1];
        
        gba_emu::emulator::run_frame();
        
        let after_cycles = emu.cycle_count;
        let after_pc = emu.cpu.r[15];
        let after_r0 = emu.cpu.r[0];
        let after_r1 = emu.cpu.r[1];
        
        eprintln!("Frame {}: cycles {}->{} (executed {}) PC 0x{:08X}->0x{:08X} R0 0x{:08X}->0x{:08X} R1 0x{:08X}->0x{:08X}",
            frame, before_cycles, after_cycles, after_cycles.wrapping_sub(before_cycles),
            before_pc, after_pc, before_r0, after_r0, before_r1, after_r1);
    }
}
