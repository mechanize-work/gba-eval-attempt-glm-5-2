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
    
    for frame in 0..700 {
        gba_emu::emulator::run_frame();
        let pc = emu.cpu.r[15];
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let r1 = emu.cpu.r[1];
        
        if pc != 0x08000190 && pc != 0x08000192 && pc != 0x08000194 {
            eprintln!("Frame {}: PC=0x{:08X} DISPCNT=0x{:04X} R1=0x{:08X} - EXITED LOOP", frame, pc, dispcnt, r1);
            // Trace a few more
            for j in 0..20 {
                let pc2 = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc2) as u32;
                gba_emu::emulator::run_frame();
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                eprintln!("  +{} PC=0x{:08X} 0x{:04X} DISPCNT=0x{:04X} halted={}", j, pc2, instr, dc, emu.cpu.halted);
                if emu.cpu.halted { break; }
            }
            break;
        }
        
        if frame % 100 == 0 {
            eprintln!("Frame {}: PC=0x{:08X} R1=0x{:08X} DISPCNT=0x{:04X}", frame, pc, r1, dispcnt);
        }
    }
}
