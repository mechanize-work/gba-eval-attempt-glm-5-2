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
        let pc = emu.cpu.r[15];
        if pc == 0x0800070C || pc == 0x0800070E || pc == 0x08000710 || pc == 0x08000714 || pc == 0x08000716 || pc == 0x08000726 {
            let instr = emu.mem.read_half(pc) as u32;
            eprintln!("[{}] PC=0x{:08X} 0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} SP={:08X} LR={:08X}",
                i, pc, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                emu.cpu.r[4], emu.cpu.r[13], emu.cpu.r[14]);
        }
        gba_emu::emulator::step_one();
        if emu.cpu.halted { break; }
    }
}
