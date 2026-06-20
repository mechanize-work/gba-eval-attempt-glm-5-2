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
    
    for i in 0..500000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        if pc >= 0x08000726 && pc <= 0x08000740 {
            eprintln!("[{}] PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} LR={:08X}",
                i, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[4], emu.cpu.r[14]);
        }
        if pc > 0x08000740 && pc < 0x08000800 && i < 226900 {
            eprintln!("[{}] PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                i, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
        }
        if emu.cpu.halted { break; }
    }
}
