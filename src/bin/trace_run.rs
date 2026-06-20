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
        let before_cc = emu.cycle_count;
        gba_emu::emulator::run_frame();
        let after_cc = emu.cycle_count;
        let pc = emu.cpu.r[15];
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        eprintln!("Frame {}: cc {}->{} (delta={}) PC=0x{:08X} DISPCNT=0x{:04X}",
            frame, before_cc, after_cc, after_cc.wrapping_sub(before_cc), pc, dispcnt);
    }
}
