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
    
    for i in 0..204660 { gba_emu::emulator::step_one(); }
    
    for j in 0..1000u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        eprintln!("[{}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} LR={:08X} SP={:08X}",
            j, pc, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[14], emu.cpu.r[13]);
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.r[15] == 0x08000190 && j > 5 {
            eprintln!("[{}] RESTART!", j+1);
            break;
        }
        if emu.cpu.r[15] == 0x08000000 || emu.cpu.r[15] == 0x080000C0 {
            eprintln!("[{}] RESET to entry!", j+1);
            break;
        }
    }
}
