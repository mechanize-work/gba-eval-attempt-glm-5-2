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
    
    // Run 3 frames (enough for init + first VBlank)
    for f in 0..3 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        eprintln!("Frame {}: DC=0x{:04X} halted={} PC=0x{:08X} [15E0]=0x{:08X}",
            f, dc, emu.cpu.halted, emu.cpu.r[15], emu.mem.read_word(0x030015E0));
    }
    
    // Now step until not halted, then trace
    if emu.cpu.halted {
        for i in 0..300000u64 {
            gba_emu::emulator::step_one();
            if !emu.cpu.halted {
                eprintln!("\nWoke up at step +{}. PC=0x{:08X}", i, emu.cpu.r[15]);
                
                // Trace 200 instructions, tracking DISPCNT and [15E0]
                let mut last_dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let mut last_15e0 = emu.mem.read_word(0x030015E0);
                let mut last_vb = emu.mem.read_word(0x03003E5C);
                
                for j in 0..1000u32 {
                    let pc = emu.cpu.r[15];
                    let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                    let v15e0 = emu.mem.read_word(0x030015E0);
                    let vb = emu.mem.read_word(0x03003E5C);
                    
                    if dc != last_dc { eprintln!("[{}] DC 0x{:04X}->0x{:04X} PC=0x{:08X}", j, last_dc, dc, pc); last_dc = dc; }
                    if v15e0 != last_15e0 { eprintln!("[{}] [15E0] 0x{:08X}->0x{:08X} PC=0x{:08X}", j, last_15e0, v15e0, pc); last_15e0 = v15e0; }
                    if vb != last_vb { eprintln!("[{}] VB_h 0x{:08X}->0x{:08X} PC=0x{:08X}", j, last_vb, vb, pc); last_vb = vb; }
                    
                    gba_emu::emulator::step_one();
                    if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
                }
                break;
            }
        }
    }
}
