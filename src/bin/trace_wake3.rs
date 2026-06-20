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
    
    // Run 1 frame (init + VBlankIntrWait halt)
    gba_emu::emulator::run_frame();
    eprintln!("After frame 0: halted={} PC=0x{:08X} vb_occ={} vbiw={}",
        emu.cpu.halted, emu.cpu.r[15], emu.vblank_occurred, emu.cpu.vblank_intr_wait);
    
    // Step through frame 1 manually, tracking VBlank
    let mut woke = false;
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        if !emu.cpu.halted && !woke {
            eprintln!("[{}] CPU woke up! PC=0x{:08X} vbiw={}", i, emu.cpu.r[15], emu.cpu.vblank_intr_wait);
            woke = true;
            
            // Trace 20 instructions
            for j in 0..20 {
                let pc = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc) as u32;
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                eprintln!("[{}] PC=0x{:08X} 0x{:04X} DC=0x{:04X} R0={:08X}", j, pc, instr, dc, emu.cpu.r[0]);
                gba_emu::emulator::step_one();
                if emu.cpu.halted { eprintln!("[{}] HALTED again", j+1); break; }
            }
            break;
        }
        
        if emu.cpu.halted && i % 100000 == 0 && i > 0 {
            eprintln!("[{}] Still halted. scanline={} vb_occ={}", i, emu.current_scanline, emu.vblank_occurred);
        }
    }
}
