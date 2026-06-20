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
    
    eprintln!("After 2 frames: halted={} vbiw={} vb_occ={}", emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred);
    eprintln!("IE=0x{:04X} IF=0x{:04X} IME={}",
        (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8),
        (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8),
        (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8));
    
    // Check what IE enables - if HBlank or VCount are enabled, they could trigger IRQ
    let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
    eprintln!("IE bits: VBlank={} HBlank={} VCount={}", ie&1, (ie>>1)&1, (ie>>2)&1);
    
    let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
    eprintln!("IF bits: VBlank={} HBlank={} VCount={}", if_&1, (if_>>1)&1, (if_>>2)&1);
    eprintln!("pending = IE & IF = 0x{:04X}", ie & if_);
}
