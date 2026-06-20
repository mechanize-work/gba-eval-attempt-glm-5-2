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
    
    let mut last_dc = 0x0080u16;
    
    for frame in 0..120 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        
        if dc != last_dc || frame < 5 || frame % 20 == 0 {
            eprintln!("Frame {}: DC=0x{:04X} halted={} PC=0x{:08X} [15E0]=0x{:08X}",
                frame, dc, emu.cpu.halted, emu.cpu.r[15],
                emu.mem.read_word(0x030015E0));
            last_dc = dc;
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
