// Trace the loop
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
    
    // Run until we reach the loop area, then trace
    for i in 0..200000u32 {
        gba_emu::emulator::step_one();
        if emu.cpu.r[15] >= 0x080001A0 && emu.cpu.r[15] <= 0x080001E0 {
            let pc = emu.cpu.r[15];
            let thumb = emu.cpu.is_thumb();
            let instr = if thumb { 
                format!("0x{:04X}", emu.mem.read_half(pc)) 
            } else { 
                format!("0x{:08X}", emu.mem.read_word(pc)) 
            };
            let dispstat = (emu.mem.io[0x04] as u16) | ((emu.mem.io[0x05] as u16) << 8);
            let vcount = (emu.mem.io[0x06] as u16) | ((emu.mem.io[0x07] as u16) << 8);
            eprintln!("[{:6}] PC=0x{:08X} {} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} DISPSTAT=0x{:04X} VCOUNT={} halted={}",
                i, pc, instr,
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7],
                dispstat, vcount, emu.cpu.halted);
            if i > 100100 {
                break;
            }
        }
    }
}
