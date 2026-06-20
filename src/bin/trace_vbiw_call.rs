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
    
    // Count SWI 0x05 calls
    let mut vbiw_count = 0u32;
    
    for i in 0..1_000_000u64 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        
        if thumb {
            let instr = emu.mem.read_half(pc);
            if (instr & 0xFF00) == 0xDF05 {
                vbiw_count += 1;
                eprintln!("[{}] VBlankIntrWait #{} at PC=0x{:08X} [15E0]=0x{:08X} DC=0x{:04X}",
                    i, vbiw_count, pc, emu.mem.read_word(0x030015E0),
                    (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
                if vbiw_count >= 10 { break; }
            }
        }
        
        gba_emu::emulator::step_one();
        
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("[{}] DISPLAY ACTIVE! DC=0x{:04X}", i, dc);
            break;
        }
    }
    
    if vbiw_count == 0 {
        eprintln!("VBlankIntrWait was never called in 1M steps!");
        eprintln!("Final PC=0x{:08X} halted={}", emu.cpu.r[15], emu.cpu.halted);
    }
}
