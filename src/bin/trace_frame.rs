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
    
    // Manually simulate run_frame for frame 0
    let target = emu.cycle_count + 280896u32;
    eprintln!("Frame 0: start cc={}, target={}", emu.cycle_count, target);
    
    let mut instr_count = 0u64;
    while emu.cycle_count < target {
        gba_emu::emulator::step_one();
        instr_count += 1;
    }
    eprintln!("Frame 0: end cc={}, instructions={}", emu.cycle_count, instr_count);
    
    // Apply the wrapping_sub
    emu.cycle_count = emu.cycle_count.wrapping_sub(280896);
    eprintln!("Frame 0: after sub cc={}", emu.cycle_count);
    
    // Frame 1
    let target1 = emu.cycle_count + 280896u32;
    eprintln!("Frame 1: start cc={}, target={}", emu.cycle_count, target1);
    
    instr_count = 0;
    while emu.cycle_count < target1 {
        gba_emu::emulator::step_one();
        instr_count += 1;
        if instr_count > 5_000_000 {
            eprintln!("Frame 1: exceeded 5M instructions, cc={}", emu.cycle_count);
            break;
        }
    }
    eprintln!("Frame 1: end cc={}, instructions={}", emu.cycle_count, instr_count);
    emu.cycle_count = emu.cycle_count.wrapping_sub(280896);
    
    // Frame 2
    let target2 = emu.cycle_count + 280896u32;
    eprintln!("Frame 2: start cc={}, target={}", emu.cycle_count, target2);
    let pc = emu.cpu.r[15];
    let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    eprintln!("Frame 2: PC=0x{:08X} DISPCNT=0x{:04X}", pc, dispcnt);
}
