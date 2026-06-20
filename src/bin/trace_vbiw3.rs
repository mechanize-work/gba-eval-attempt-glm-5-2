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
    
    // Track [0x030015E0] changes and VBlankIntrWait calls
    let mut last_15e0 = 0u32;
    
    for i in 0..500000u64 {
        let pc = emu.cpu.r[15];
        let instr = if emu.cpu.is_thumb() { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let is_vbiw = (instr & 0xFF00) == 0xDF05;
        
        if is_vbiw {
            eprintln!("[{}] VBlankIntrWait at PC=0x{:08X} [15E0]=0x{:08X}", i, pc, emu.mem.read_word(0x030015E0));
        }
        
        gba_emu::emulator::step_one();
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [0x030015E0] = 0x{:08X} -> 0x{:08X} at PC=0x{:08X}", i, last_15e0, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
        }
        
        if emu.cpu.halted && i > 226000 {
            // Don't break on halt - the VBlankIntrWait should wake up
        }
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != 0x0080 {
            eprintln!("[{}] DISPCNT=0x{:04X} at PC=0x{:08X}", i, dc, emu.cpu.r[15]);
            break;
        }
    }
    eprintln!("Final [15E0]=0x{:08X}", last_15e0);
}
