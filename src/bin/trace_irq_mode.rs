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
    
    // Run 2 frames to get past init
    for _ in 0..2 { gba_emu::emulator::run_frame(); }
    
    // Trace 50 instructions showing mode
    for j in 0..50 {
        let pc = emu.cpu.r[15];
        let mode = emu.cpu.cpsr & 0x1F;
        let mode_str = match mode { 0x10=>"USR",0x11=>"FIQ",0x12=>"IRQ",0x13=>"SVC",0x1F=>"SYS",_=>&"???" };
        let thumb = emu.cpu.is_thumb();
        let irq_proc = emu.irq_processing;
        let irq_bits = emu.irq_pending_bits;
        
        eprintln!("[{}] PC=0x{:08X} {} {} irq_proc={} irq_bits=0x{:04X} IE=0x{:04X} IF=0x{:04X} IME={}",
            j, pc, mode_str, if thumb {"T"} else {"A"}, irq_proc, irq_bits,
            (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8),
            (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8),
            (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8));
        
        gba_emu::emulator::step_one();
    }
}
