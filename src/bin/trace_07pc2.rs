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
    
    // Run 2 frames
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    // Run frame 2, tracking PC
    let target = emu.cycle_count.wrapping_add(280896);
    let mut count = 0u64;
    let mut last_range = 0u32;
    
    while emu.cycle_count < target && count < 500000 {
        emu.check_and_handle_interrupts();
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            emu.advance_hardware(1);
        } else {
            emu.execute_one();
            emu.check_and_handle_interrupts();
        }
        count += 1;
        
        let pc = emu.cpu.r[15];
        let range = pc >> 24;
        if range != last_range {
            eprintln!("[{}] PC=0x{:08X} (range 0x{:02X}) LR=0x{:08X} halted={}",
                count, pc, range, emu.cpu.r[14], emu.cpu.halted);
            last_range = range;
        }
    }
}
