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
    
    // Run 2 frames
    for f in 0..2 {
        gba_emu::emulator::run_frame();
        eprintln!("Frame {}: PC=0x{:08X} halted={} vb_occ={} vbiw={}",
            f, emu.cpu.r[15], emu.cpu.halted, emu.vblank_occurred, emu.cpu.vblank_intr_wait);
    }
    
    // Now step through frame 2 manually
    let mut woke = false;
    for i in 0..400000u64 {
        gba_emu::emulator::step_one();
        
        if !emu.cpu.halted && !woke && i > 100000 {
            eprintln!("[{}] CPU woke up! PC=0x{:08X} vbiw={}", i, emu.cpu.r[15], emu.cpu.vblank_intr_wait);
            woke = true;
            
            // Check [0x030015E0] and [0x03003C18]
            let v15e0 = emu.mem.read_word(0x030015E0);
            let v3c18 = emu.mem.read_word(0x03003C18);
            let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            eprintln!("  [0x030015E0]=0x{:08X} [0x03003C18]=0x{:08X} DC=0x{:04X}", v15e0, v3c18, dc);
            
            // Trace 30 instructions
            for j in 0..30 {
                let pc = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc) as u32;
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let v15e0 = emu.mem.read_word(0x030015E0);
                eprintln!("[{:2}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} [15E0]={:08X} R0={:08X} R1={:08X}",
                    j, pc, instr, dc, v15e0, emu.cpu.r[0], emu.cpu.r[1]);
                gba_emu::emulator::step_one();
                if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
            }
            break;
        }
        
        if emu.cpu.halted && i % 100000 == 0 && i > 0 {
            eprintln!("[{}] Still halted. scanline={} vb_occ={}", i, emu.current_scanline, emu.vblank_occurred);
        }
    }
}
