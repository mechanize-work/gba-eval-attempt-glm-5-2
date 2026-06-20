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
    
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        if emu.mem.read_word(0x030015E0) != 0 {
            let mut last_dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
            
            for j in 0..2000u32 {
                let pc = emu.cpu.r[15];
                let thumb = emu.cpu.is_thumb();
                let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
                let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
                let is_swi = if thumb { (instr & 0xFF00) == 0xDF00 } else { (instr >> 24) == 0xEF };
                let is_bl = (instr & 0xF800) == 0xF000;
                
                if dc != last_dc { eprintln!("[{}] DC 0x{:04X}->0x{:04X} PC=0x{:08X}", j, last_dc, dc, pc); last_dc = dc; }
                
                if is_swi {
                    let swi_str = if thumb { format!("SWI={}", instr & 0xFF) } else { format!("ARM_SWI=0x{:X}", instr & 0xFFFFFF) };
                    eprintln!("[{}] PC=0x{:08X} {} R0={:08X} R1={:08X} R2={:08X}", j, pc, swi_str, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
                } else if is_bl {
                    eprintln!("[{}] PC=0x{:08X} BL_HI R0={:08X} R1={:08X} R2={:08X}", j, pc, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
                } else if pc >= 0x08000726 && pc <= 0x08000740 {
                    eprintln!("[{}] PC=0x{:08X} 0x{:X} R0={:08X} R1={:08X} R2={:08X}", j, pc, instr, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2]);
                }
                
                gba_emu::emulator::step_one();
                if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
            }
            break;
        }
    }
}
