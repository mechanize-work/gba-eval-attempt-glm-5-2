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
    
    // Step to just before SWI 0x0C (step 226152)
    for i in 0..226151 {
        gba_emu::emulator::step_one();
    }
    
    eprintln!("Before SWI 0x0C:");
    eprintln!("  R0=0x{:08X} R1=0x{:08X} R2=0x{:08X}", emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
    
    // Execute the SWI (2 steps to get past it)
    gba_emu::emulator::step_one();
    
    eprintln!("After SWI 0x0C:");
    
    // Check IWRAM after - compare with ROM
    let src_base = 0x080125B4u32; // word-aligned source
    let dst_base = 0x03000000u32;
    let mut all_match = true;
    for i in 0..10 {
        let rom_val = emu.mem.read_word(src_base.wrapping_add(i * 4));
        let iwram_val = emu.mem.read_word(dst_base.wrapping_add(i * 4));
        let match_str = if rom_val == iwram_val { "OK" } else { "MISMATCH" };
        if rom_val != iwram_val { all_match = false; }
        eprintln!("  [{}] ROM=0x{:08X} IWRAM=0x{:08X} {}", i, rom_val, iwram_val, match_str);
    }
    eprintln!("All match: {}", all_match);
}
