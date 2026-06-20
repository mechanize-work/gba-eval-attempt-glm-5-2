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
    
    // Step through 300K instructions, logging PC changes to new 256-byte ranges
    let mut last_range = 0u32;
    for i in 0..300_000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        let range = pc & 0xFFFF_FF00;
        
        if range != last_range {
            let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            let halted = emu.cpu.halted;
            let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
            eprintln!("[{}] PC=0x{:08X} DC=0x{:04X} h={} IME={} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}",
                i, pc, dispcnt, halted, ime, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[13], emu.cpu.r[14]);
            last_range = range;
        }
        
        // Check for SWI
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        if thumb && (instr & 0xFF00) == 0xDF00 {
            eprintln!("[{}] SWI {} at PC=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                i, instr & 0xFF, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
        }
        
        if emu.cpu.halted {
            eprintln!("[{}] CPU HALTED at PC=0x{:08X}", i, pc);
            break;
        }
    }
}
