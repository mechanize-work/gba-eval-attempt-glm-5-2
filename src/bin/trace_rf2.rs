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
    
    // Reproduce run_frame manually
    let target = emu.cycle_count.wrapping_add(280896u32);
    eprintln!("Start: cc={}, target={}", emu.cycle_count, target);
    
    let mut count = 0u64;
    while emu.cycle_count < target && count < 2_000_000 {
        // Check interrupts
        let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
        let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
        let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
        if ime != 0 && (ie & if_) != 0 {
            eprintln!("[{}] IRQ pending! ie=0x{:04X} if=0x{:04X} ime={}", count, ie, if_, ime);
        }
        
        if emu.cpu.halted {
            emu.cycle_count = emu.cycle_count.wrapping_add(1);
            count += 1;
        } else {
            gba_emu::emulator::step_one();
            count += 1;
        }
    }
    eprintln!("End: cc={}, instructions={}", emu.cycle_count, count);
    eprintln!("PC=0x{:08X}", emu.cpu.r[15]);
}
