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
    
    // Track [0x03003E5C] (VBlank handler in vector table) and [0x030015E0]
    let mut last_vb = 0u32;
    let mut last_15e0 = 0u32;
    let mut last_dc = 0x0080u16;
    let mut last_ie = 0u16;
    let mut last_ime = 0u16;
    let mut last_7ffc = 0u32;
    
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        
        let vb = emu.mem.read_word(0x03003E5C);
        let v15e0 = emu.mem.read_word(0x030015E0);
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        let v7ffc = emu.mem.read_word(0x03007FFC);
        
        if vb != last_vb || v15e0 != last_15e0 || dc != last_dc || ie != last_ie || ime != last_ime || v7ffc != last_7ffc {
            eprintln!("[{}] PC=0x{:08X} DC=0x{:04X} VB_h=0x{:08X} [15E0]=0x{:08X} IE=0x{:04X} IME={} [7FFC]=0x{:08X}",
                i, emu.cpu.r[15], dc, vb, v15e0, ie, ime, v7ffc);
            last_vb = vb;
            last_15e0 = v15e0;
            last_dc = dc;
            last_ie = ie;
            last_ime = ime;
            last_7ffc = v7ffc;
        }
        
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("  DISPLAY ACTIVE at step {}!", i);
            break;
        }
    }
}
