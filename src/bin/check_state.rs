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
    
    for f in 0..10 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let h = emu.cpu.halted;
        let pc = emu.cpu.r[15];
        let v15e0 = emu.mem.read_word(0x030015E0);
        let v7ffc = emu.mem.read_word(0x03007FFC);
        eprintln!("Frame {}: DC=0x{:04X} h={} PC=0x{:08X} [15E0]=0x{:08X} [7FFC]=0x{:08X}",
            f, dc, h, pc, v15e0, v7ffc);
    }
}
