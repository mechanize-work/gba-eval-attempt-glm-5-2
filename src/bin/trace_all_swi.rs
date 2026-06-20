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
    
    for i in 0..300_000u64 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        
        let is_swi = if thumb { (instr & 0xFF00) == 0xDF00 } else { (instr >> 24) == 0xEF };
        
        if is_swi {
            let swi_num = if thumb { instr & 0xFF } else { instr & 0xFFFFFF };
            eprintln!("[{}] SWI 0x{:02X} at PC=0x{:08X} ({}) R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                i, swi_num, pc, if thumb {"T"} else {"A"},
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
        }
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.halted {
            break;
        }
    }
}
