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
    
    // Track DISPCNT, [0x030015E0], [0x03003E5C], and [0x03003C18] changes
    let mut last_dc = 0x0080u16;
    let mut last_15e0 = 0u32;
    let mut last_vb = 0u32;
    let mut last_3c18 = 0u32;
    
    for frame in 0..30 {
        gba_emu::emulator::run_frame();
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let v15e0 = emu.mem.read_word(0x030015E0);
        let vb = emu.mem.read_word(0x03003E5C);
        let v3c18 = emu.mem.read_word(0x03003C18);
        
        if dc != last_dc || v15e0 != last_15e0 || vb != last_vb || v3c18 != last_3c18 || frame < 5 {
            eprintln!("Frame {}: DC=0x{:04X} [15E0]=0x{:08X} VB_h=0x{:08X} [3C18]=0x{:08X} halted={} PC=0x{:08X}",
                frame, dc, v15e0, vb, v3c18, emu.cpu.halted, emu.cpu.r[15]);
            last_dc = dc;
            last_15e0 = v15e0;
            last_vb = vb;
            last_3c18 = v3c18;
        }
        
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("  DISPLAY ACTIVE!");
            break;
        }
    }
}
