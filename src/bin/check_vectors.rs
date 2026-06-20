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
    
    // Check IRQ vector table at 0x03003E5C
    eprintln!("IRQ vector table at 0x03003E5C:");
    let names = ["VBlank", "HBlank", "VCount", "Timer0", "Timer1", "Timer2", "Timer3",
                 "SIO", "DMA0", "DMA1", "DMA2", "DMA3", "Keypad", "GamePak"];
    for i in 0..14usize {
        let addr = (0x03003E5C + i * 4) as u32;
        let val = emu.mem.read_word(addr);
        eprintln!("  [{}][0x{:08X}] = 0x{:08X} ({})", i, addr, val, names.get(i).unwrap_or(&"?"));
    }
    
    // Check what the default handler at 0x080133C0 does
    eprintln!("\nDefault handler at 0x080133C0:");
    let val = emu.mem.read_half(0x080133C0);
    eprintln!("  0x080133C0: 0x{:04X} (BX LR)", val);
    
    // Check [0x030015E0] and DISPCNT
    eprintln!("\n[0x030015E0] = 0x{:08X}", emu.mem.read_word(0x030015E0));
    eprintln!("DISPCNT = 0x{:04X}", (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8));
    eprintln!("PC = 0x{:08X} halted = {}", emu.cpu.r[15], emu.cpu.halted);
    
    // Check if the game registered a VBlank handler
    // The vector table at 0x03003E5C[0] = VBlank handler
    // If it's 0x080133C1 (default), no real handler is registered
    let vb_handler = emu.mem.read_word(0x03003E5C);
    eprintln!("\nVBlank handler = 0x{:08X}", vb_handler);
    if vb_handler == 0x080133C1 {
        eprintln!("  -> DEFAULT handler (no real VBlank handler registered)");
    }
}
