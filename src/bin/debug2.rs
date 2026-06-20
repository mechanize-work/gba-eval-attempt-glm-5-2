// More detailed debug
use std::fs;

fn main() {
    let rom_data = fs::read("dev-roms/anguna.gba").expect("Failed to read ROM");
    eprintln!("ROM size: {} bytes", rom_data.len());

    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());

    let emu = gba_emu::emulator::get_emu();
    
    // Step instructions and check state periodically
    for i in 0..500000u32 {
        gba_emu::emulator::step_one();
        
        if i % 50000 == 0 {
            let pc = emu.cpu.r[15];
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
            let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
            let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
            let halted = emu.cpu.halted;
            let thumb = emu.cpu.is_thumb();
            eprintln!("[{:6}] PC=0x{:08X} THUMB={} halted={} DISPCNT=0x{:04X} IME={} IE=0x{:04X} IF=0x{:04X} cycles={}",
                i, pc, thumb, halted, dispcnt, ime, ie, if_, emu.cycle_count);
        }
        
        if emu.cpu.halted && i > 100000 {
            eprintln!("CPU halted at PC=0x{:08X} after {} steps", emu.cpu.r[15], i);
            eprintln!("  IME={} IE=0x{:04X} IF=0x{:04X}", 
                (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8),
                (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8),
                (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8));
            break;
        }
    }
    
    // Run one frame
    gba_emu::emulator::run_frame();
    
    let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    let bg0cnt = (emu.mem.io[0x08] as u16) | ((emu.mem.io[0x09] as u16) << 8);
    eprintln!("\nAfter run_frame:");
    eprintln!("  DISPCNT=0x{:04X} mode={} bg_en=0x{:X}", dispcnt, dispcnt & 7, (dispcnt >> 8) & 0xF);
    eprintln!("  BG0CNT=0x{:04X}", bg0cnt);
    eprintln!("  cycle_count={}", emu.cycle_count);
    
    // Check framebuffer
    let fb = &emu.ppu.framebuffer;
    let non_zero = fb.iter().filter(|p| **p != 0).count();
    eprintln!("  Framebuffer: {} non-zero pixels", non_zero);
    for i in 0..5 {
        if fb[i] != 0 {
            eprintln!("  fb[{}]=0x{:08X}", i, fb[i]);
        }
    }
    
    // Check palette
    let palette_bg0 = (emu.mem.palette[0] as u16) | ((emu.mem.palette[1] as u16) << 8);
    eprintln!("  BG palette[0]=0x{:04X}", palette_bg0);
}
