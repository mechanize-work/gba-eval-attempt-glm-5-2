// Debug interrupt handling and VBlank
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
    
    // Run frames and check when DISPCNT changes or IME/IE gets set
    for frame in 0..20 {
        let before_dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let before_ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let before_ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let before_if = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let before_halted = emu.cpu.halted;
        let before_pc = emu.cpu.r[15];
        
        gba_emu::emulator::run_frame();
        
        let after_dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let after_ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let after_ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let after_if = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let after_halted = emu.cpu.halted;
        let after_pc = emu.cpu.r[15];
        
        let dispstat = (emu.mem.io[0x04] as u16) | ((emu.mem.io[0x05] as u16) << 8);
        let vcount = (emu.mem.io[0x06] as u16) | ((emu.mem.io[0x07] as u16) << 8);
        
        eprintln!("Frame {}: PC 0x{:08X}->0x{:08X} halted {}->{} DISPCNT 0x{:04X}->0x{:04X} IME {}->{} IE 0x{:04X}->0x{:04X} IF 0x{:04X}->0x{:04X} DISPSTAT=0x{:04X} VCOUNT={}",
            frame, before_pc, after_pc, before_halted, after_halted,
            before_dispcnt, after_dispcnt,
            before_ime, after_ime, before_ie, after_ie, before_if, after_if,
            dispstat, vcount);
    }
}
