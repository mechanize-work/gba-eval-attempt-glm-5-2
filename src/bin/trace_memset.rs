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
    
    // Run until we reach the memset loop
    for i in 0..1_000_000u32 {
        gba_emu::emulator::step_one();
        if emu.cpu.r[15] == 0x08000190 {
            eprintln!("Reached memset loop at step {}, cycle_count={}", i, emu.cycle_count);
            eprintln!("R0=0x{:08X} R1=0x{:08X} R2=0x{:08X}", emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
            
            // Count how many iterations until R1 reaches 0
            let r1_start = emu.cpu.r[1];
            let iterations_needed = r1_start / 4;
            eprintln!("R1=0x{:08X}, iterations needed: {} ({} cycles at 6/iter)", 
                r1_start, iterations_needed, iterations_needed * 6);
            
            // Now step until R1 reaches 0 or we exit the loop
            let mut last_r1 = emu.cpu.r[1];
            for j in 0..1_000_000u32 {
                gba_emu::emulator::step_one();
                let pc = emu.cpu.r[15];
                if pc != 0x08000190 && pc != 0x08000192 && pc != 0x08000194 {
                    eprintln!("Exited memset loop at step {}! PC=0x{:08X} R1=0x{:08X}", 
                        j, pc, emu.cpu.r[1]);
                    break;
                }
                if emu.cpu.r[1] == 0 {
                    eprintln!("R1 reached 0 at step {}, PC=0x{:08X}", j, pc);
                }
            }
            break;
        }
    }
}
