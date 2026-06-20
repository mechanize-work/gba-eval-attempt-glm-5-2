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
    
    for frame in 0..30 {
        gba_emu::emulator::run_frame();
        let pc = emu.cpu.r[15];
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let halted = emu.cpu.halted;
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let r1 = emu.cpu.r[1];
        
        eprintln!("Frame {:2}: PC=0x{:08X} DISPCNT=0x{:04X} halted={} IME={} IE=0x{:04X} IF=0x{:04X} R1=0x{:08X}",
            frame, pc, dispcnt, halted, ime, ie, if_, r1);
        
        if dispcnt != 0x0080 && dispcnt != 0 {
            eprintln!("  DISPCNT changed! Game progressing.");
        }
        
        if halted {
            eprintln!("  CPU halted!");
        }
    }
}
