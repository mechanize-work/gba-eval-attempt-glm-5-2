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
    
    // Step to 282190, tracking R6 changes
    let mut last_r6 = 0u32;
    for i in 0..282190 {
        gba_emu::emulator::step_one();
        let r6 = emu.cpu.r[6];
        if r6 != last_r6 {
            if i > 282000 {
                eprintln!("[{}] R6 changed: 0x{:08X} -> 0x{:08X} PC=0x{:08X}",
                    i, last_r6, r6, emu.cpu.r[15]);
            }
            last_r6 = r6;
        }
    }
    
    eprintln!("\nAt step 282190:");
    eprintln!("  R6=0x{:08X}", emu.cpu.r[6]);
    eprintln!("  R2=0x{:08X}", emu.cpu.r[2]);
    
    // Read what's at [R6]
    let r6 = emu.cpu.r[6];
    let val = emu.mem.read_word(r6);
    eprintln!("  [R6]=[0x{:08X}]=0x{:08X}", r6, val);
    
    // Check what's in ROM around the expected function address
    // If the function should be at 0x08006E6C, let's check
    eprintln!("\n  ROM[0x08006E0E] = 0x{:04X} (LDR R2, [R6])", emu.mem.read_rom_half(0x08006E0E));
    
    // Trace backwards from the BX to find where R6 was loaded
    eprintln!("\nTracing R6 history (last 20 changes before step 282190):");
}
