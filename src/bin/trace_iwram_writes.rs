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
    
    // Track writes to IWRAM 0x03000EF0-0x03000F50
    let watch_start = 0x03000EF0u32;
    let watch_end = 0x03000F50u32;
    let mut last_vals: Vec<u8> = emu.mem.iwram[0xEF0..0xF50].to_vec();
    
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        
        // Check if any watched bytes changed
        for j in 0..0x60 {
            let iwram_offset = 0xEF0 + j;
            if emu.mem.iwram[iwram_offset] != last_vals[j] {
                let addr = 0x03000000 + iwram_offset as u32;
                eprintln!("[{}] [0x{:08X}] = 0x{:02X} -> 0x{:02X} at PC=0x{:08X}",
                    i, addr, last_vals[j], emu.mem.iwram[iwram_offset], emu.cpu.r[15]);
                last_vals[j] = emu.mem.iwram[iwram_offset];
            }
        }
        
        if emu.cpu.halted && i > 226000 {
            // Check if the IRQ handler fires and modifies IWRAM
        }
    }
}
