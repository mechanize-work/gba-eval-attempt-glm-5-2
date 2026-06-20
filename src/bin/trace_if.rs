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
    
    for frame in 0..60 {
        gba_emu::emulator::run_frame();
        
        let pc = emu.cpu.r[15];
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let irq_if = emu.irq.if_;
        let halted = emu.cpu.halted;
        let dispstat = (emu.mem.io[0x04] as u16) | ((emu.mem.io[0x05] as u16) << 8);
        let vcount = (emu.mem.io[0x06] as u16) | ((emu.mem.io[0x07] as u16) << 8);
        
        if frame < 5 || frame % 10 == 0 || dispcnt != 0 || ime != 0 || halted {
            eprintln!("Frame {}: PC=0x{:08X} DISPCNT=0x{:04X} IME={} IE=0x{:04X} IF=0x{:04X}(irq=0x{:04X}) halted={} DISPSTAT=0x{:04X} VCOUNT={}",
                frame, pc, dispcnt, ime, ie, if_, irq_if, halted, dispstat, vcount);
        }
        
        if dispcnt != 0 {
            break;
        }
    }
}
