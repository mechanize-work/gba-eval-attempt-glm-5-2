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
    
    // Run 3 frames to get past init
    for _ in 0..3 { gba_emu::emulator::run_frame(); }
    
    // Now trace 100 instructions, showing IRQ handler flow
    for j in 0..200u32 {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        let instr: u32 = if thumb { emu.mem.read_half(pc) as u32 } else { emu.mem.read_word(pc) };
        let mode = emu.cpu.cpsr & 0x1F;
        let mode_str = match mode { 0x10=>"USR",0x11=>"FIQ",0x12=>"IRQ",0x13=>"SVC",0x1F=>"SYS",_=>&"???" };
        
        // Show when in IRQ/SVC mode or in IWRAM/BIOS
        if pc < 0x04000000 || mode != 0x1F || j < 5 {
            eprintln!("[{:3}] PC=0x{:08X} 0x{:X} {} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}",
                j, pc, instr, mode_str, emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                emu.cpu.r[13], emu.cpu.r[14]);
        }
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.halted { eprintln!("[{}] HALTED", j+1); break; }
    }
}
