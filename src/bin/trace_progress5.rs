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
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let halted = emu.cpu.halted;
        let v15e0 = emu.mem.read_word(0x030015E0);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        
        if dc != 0x0080 || frame < 3 || frame % 10 == 0 {
            eprintln!("Frame {}: DC=0x{:04X} h={} PC=0x{:08X} [15E0]={:08X} IE=0x{:04X} IME={}",
                frame, dc, halted, emu.cpu.r[15], v15e0, ie, ime);
        }
        
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("  DISPLAY ACTIVE!");
            let fb = &emu.ppu.framebuffer;
            let non_black = fb.iter().filter(|p| **p != 0 && **p != 0xFF000000).count();
            eprintln!("  Non-black pixels: {}", non_black);
            break;
        }
    }
}
