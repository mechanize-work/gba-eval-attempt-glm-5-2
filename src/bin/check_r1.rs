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
    
    // Track when PC reaches 0x08000190 (memset loop) and show R1
    let mut last_r1 = 0u32;
    let mut in_memset = false;
    let mut memset_count = 0u32;
    
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        
        if pc == 0x08000190 && !in_memset {
            in_memset = true;
            memset_count += 1;
            eprintln!("[{}] Enter memset #{}: R0=0x{:08X} R1=0x{:08X} R2=0x{:08X}",
                i, memset_count, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
        }
        
        if pc != 0x08000190 && pc != 0x08000192 && pc != 0x08000194 && pc != 0x08000196 && in_memset {
            in_memset = false;
            eprintln!("[{}] Exit memset #{}: PC=0x{:08X} R1=0x{:08X}",
                i, memset_count, pc, emu.cpu.r[1]);
        }
        
        if memset_count >= 5 && !in_memset {
            break;
        }
    }
}
