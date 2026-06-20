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
    
    // Run enough steps to get past init
    for i in 0..300000u64 {
        gba_emu::emulator::step_one();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        if dc != 0x0080 && dc != 0x0000 {
            eprintln!("Step {}: DC=0x{:04X} scan={} snap_dc=0x{:04X}", i, dc, emu.current_scanline, emu.ppu.snap_dispcnt);
            break;
        }
    }
    
    // Now run frames and check snapshot vs live DISPCNT
    for f in 0..10 {
        gba_emu::emulator::run_frame();
        let dc = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let snap = emu.ppu.snap_dispcnt;
        eprintln!("Frame {}: DC=0x{:04X} snap=0x{:04X} scan={} halted={}", 
            f, dc, snap, emu.current_scanline, emu.cpu.halted);
    }
}
