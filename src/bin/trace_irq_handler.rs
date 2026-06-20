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
    
    // Run 2 frames to reach VBlankIntrWait halt
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    eprintln!("Halted at PC=0x{:08X} vbiw={}", emu.cpu.r[15], emu.cpu.vblank_intr_wait);
    eprintln!("VB handler = 0x{:08X}", emu.mem.read_word(0x03003E5C));
    eprintln!("[15E0] = 0x{:08X}", emu.mem.read_word(0x030015E0));
    
    // Step until CPU wakes, trace the IRQ handler
    let mut last_15e0 = emu.mem.read_word(0x030015E0);
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        
        let v15e0 = emu.mem.read_word(0x030015E0);
        if v15e0 != last_15e0 {
            eprintln!("[{}] [15E0] 0x{:08X} -> 0x{:08X} at PC=0x{:08X}", i, last_15e0, v15e0, emu.cpu.r[15]);
            last_15e0 = v15e0;
        }
        
        if !emu.cpu.halted && i > 1000 {
            eprintln!("[{}] CPU woke up! PC=0x{:08X}", i, emu.cpu.r[15]);
            
            // Trace 100 instructions
            for j in 0..100u32 {
                let pc = emu.cpu.r[15];
                let thumb = emu.cpu.is_thumb();
                let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
                let mode = emu.cpu.cpsr & 0x1F;
                let mode_str = match mode { 0x10=>"USR",0x11=>"FIQ",0x12=>"IRQ",0x13=>"SVC",0x1F=>"SYS",_=>"??" };
                let v15e0 = emu.mem.read_word(0x030015E0);
                
                // Show VBlank handler execution
                if pc == 0x08008AA0 || pc == 0x08008AA1 || pc == 0x08008AA2 || 
                   (pc >= 0x08008A90 && pc <= 0x08008AB0) || v15e0 != last_15e0 || j < 5 {
                    eprintln!("[{}] PC=0x{:08X} 0x{:X} {}{} [15E0]=0x{:08X} R0={:08X}",
                        j, pc, instr, mode_str, if thumb {"T"} else {"A"}, v15e0, emu.cpu.r[0]);
                }
                
                gba_emu::emulator::step_one();
                if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
            }
            break;
        }
    }
}
