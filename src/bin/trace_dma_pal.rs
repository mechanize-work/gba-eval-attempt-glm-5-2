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
    
    eprintln!("ROM at 0x0802EC10:");
    for i in 0..8 {
        let val = emu.mem.read_rom_word(0x0802EC10 + i * 4);
        eprintln!("  [0x{:08X}] = 0x{:08X}", 0x0802EC10 + i*4, val);
    }
    
    for i in 0..226528 { gba_emu::emulator::step_one(); }
    
    let pal_before = emu.mem.palette[0];
    eprintln!("\nPalette[0] before DMA: 0x{:02X}", pal_before);
    
    gba_emu::emulator::step_one();
    
    let pal_after = emu.mem.palette[0];
    eprintln!("Palette[0] after DMA: 0x{:02X}", pal_after);
    
    eprintln!("\nPalette after DMA (first 32 bytes):");
    for i in 0..32 {
        eprint!("{:02X} ", emu.mem.palette[i]);
        if (i+1) % 16 == 0 { eprintln!(); }
    }
    
    eprintln!("\nTest: write_word(0x05000000, 0x12345678)");
    emu.mem.write_word(0x05000000, 0x12345678);
    let test = (emu.mem.palette[0] as u32) | ((emu.mem.palette[1] as u32) << 8)
        | ((emu.mem.palette[2] as u32) << 16) | ((emu.mem.palette[3] as u32) << 24);
    eprintln!("Result: palette[0..3] = 0x{:08X}", test);
}
