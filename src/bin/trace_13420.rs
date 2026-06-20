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
    
    // Step to when PC reaches 0x08013420 (called from 0x0800032A)
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let pc = emu.cpu.r[15];
        if pc == 0x08013420 {
            eprintln!("[{}] At 0x08013420! R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}",
                i, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3], emu.cpu.r[13], emu.cpu.r[14]);
            
            // Trace 80 instructions and track [0x030015E0]
            let mut last_15e0 = emu.mem.read_word(0x030015E0);
            for j in 0..80u32 {
                let pc2 = emu.cpu.r[15];
                let instr = emu.mem.read_half(pc2) as u32;
                let v15e0 = emu.mem.read_word(0x030015E0);
                
                if v15e0 != last_15e0 || j < 10 || (instr & 0xFF00) == 0xDF00 {
                    eprintln!("[{:2}] PC=0x{:08X} 0x{:04X} [15E0]=0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
                        j, pc2, instr, v15e0, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3]);
                    last_15e0 = v15e0;
                }
                
                gba_emu::emulator::step_one();
            }
            break;
        }
    }
}
