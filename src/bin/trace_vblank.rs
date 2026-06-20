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
    
    // Run 5 frames
    for _ in 0..5 { gba_emu::emulator::run_frame(); }
    
    // Check state
    let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
    let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
    let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
    let cpsr = emu.cpu.cpsr;
    let i_flag = (cpsr & 0x80) != 0;
    let halted = emu.cpu.halted;
    
    eprintln!("After 5 frames:");
    eprintln!("  PC=0x{:08X} halted={}", emu.cpu.r[15], halted);
    eprintln!("  IME={} IE=0x{:04X} IF=0x{:04X}", ime, ie, if_);
    eprintln!("  CPSR=0x{:08X} I_flag={}", cpsr, i_flag);
    eprintln!("  DISPCNT=0x{:04X}", (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
    
    // Run one more frame and see if VBlank fires
    for frame in 5..20 {
        gba_emu::emulator::run_frame();
        let pc = emu.cpu.r[15];
        let halted = emu.cpu.halted;
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        eprintln!("Frame {}: PC=0x{:08X} halted={} DC=0x{:04X} IME={} IE=0x{:04X} IF=0x{:04X} I={}",
            frame, pc, halted, dc, ime, ie, if_, (emu.cpu.cpsr & 0x80) != 0);
    }
}
