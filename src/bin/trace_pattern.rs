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
    
    // Track all unique PCs visited across many frames
    use std::collections::HashSet;
    let mut visited: HashSet<u32> = HashSet::new();
    
    for frame in 0..200 {
        gba_emu::emulator::run_frame();
        let pc = emu.cpu.r[15];
        let range = pc & 0xFFFFFF00;
        visited.insert(range);
    }
    
    // Print all unique PC ranges
    let mut ranges: Vec<u32> = visited.into_iter().collect();
    ranges.sort();
    eprintln!("Unique PC ranges visited in 200 frames:");
    for r in ranges {
        eprintln!("  0x{:08X}", r);
    }
}
