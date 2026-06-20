// Check when DISPCNT changes
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
    
    for frame in 0..300 {
        gba_emu::emulator::run_frame();
        
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let halted = emu.cpu.halted;
        
        if dispcnt != 0 || ime != 0 || ie != 0 || halted {
            eprintln!("Frame {}: DISPCNT=0x{:04X} IME={} IE=0x{:04X} halted={} PC=0x{:08X}",
                frame, dispcnt, ime, ie, halted, emu.cpu.r[15]);
        }
        
        if dispcnt != 0 && frame > 5 {
            // Found it! Dump more state
            let bg0cnt = (emu.mem.io[0x08] as u16) | ((emu.mem.io[0x09] as u16) << 8);
            let bg1cnt = (emu.mem.io[0x0A] as u16) | ((emu.mem.io[0x0B] as u16) << 8);
            let bg2cnt = (emu.mem.io[0x0C] as u16) | ((emu.mem.io[0x0D] as u16) << 8);
            let bg3cnt = (emu.mem.io[0x0E] as u16) | ((emu.mem.io[0x0F] as u16) << 8);
            eprintln!("  BG0CNT=0x{:04X} BG1CNT=0x{:04X} BG2CNT=0x{:04X} BG3CNT=0x{:04X}",
                bg0cnt, bg1cnt, bg2cnt, bg3cnt);
            
            // Check palette
            let pal0 = (emu.mem.palette[0] as u16) | ((emu.mem.palette[1] as u16) << 8);
            eprintln!("  BG pal[0]=0x{:04X}", pal0);
            
            // Check framebuffer
            let fb = &emu.ppu.framebuffer;
            let non_black = fb.iter().filter(|p| **p != 0 && **p != 0xFF000000).count();
            eprintln!("  Non-black pixels: {}", non_black);
            break;
        }
    }
}
