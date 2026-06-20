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
    
    // Track all IO register writes between CpuFastSet and VBlankIntrWait
    let mut last_io = [0u8; 0x400];
    
    for i in 0..226834u64 {
        gba_emu::emulator::step_one();
        
        if i >= 226152 {
            // Check for IO writes
            for j in 0..0x400 {
                if emu.mem.io[j] != last_io[j] {
                    let addr = 0x04000000 + j as u32;
                    eprintln!("[{}] IO[0x{:08X}] = 0x{:02X} -> 0x{:02X} at PC=0x{:08X}",
                        i, addr, last_io[j], emu.mem.io[j], emu.cpu.r[15]);
                    last_io[j] = emu.mem.io[j];
                }
            }
        }
    }
}
