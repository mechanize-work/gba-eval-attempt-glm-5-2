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
    
    // Run 2 frames using run_frame
    gba_emu::emulator::run_frame();
    gba_emu::emulator::run_frame();
    
    // Save state
    let s_halted = emu.cpu.halted;
    let s_vbiw = emu.cpu.vblank_intr_wait;
    let s_vb_occ = emu.vblank_occurred;
    let s_cc = emu.cycle_count;
    let s_scan = emu.current_scanline;
    let s_15e0 = emu.mem.read_word(0x030015E0);
    let s_irq_proc = emu.irq_processing;
    let s_irq_bits = emu.irq_pending_bits;
    
    eprintln!("State after 2 rf: halted={} vbiw={} vb_occ={} cc={} scan={} [15E0]={} irq_proc={} irq_bits={}",
        s_halted, s_vbiw, s_vb_occ, s_cc, s_scan, s_15e0, s_irq_proc, s_irq_bits);
    
    // Run frame 3 using run_frame - check if VBlankIntrWait fires
    gba_emu::emulator::run_frame();
    
    eprintln!("After rf3: halted={} vbiw={} vb_occ={} [15E0]={} irq_proc={} irq_bits={}",
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.vblank_occurred,
        emu.mem.read_word(0x030015E0), emu.irq_processing, emu.irq_pending_bits);
}
