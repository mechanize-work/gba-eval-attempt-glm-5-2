fn rd16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off+1]])
}
fn rd32(buf: &[u8], off: usize) -> u32 {
    u16::from_le_bytes([buf[off], buf[off+1]]) as u32 | ((u16::from_le_bytes([buf[off+2], buf[off+3]]) as u32) << 16)
}

fn main() {
    let rom_data = std::fs::read("dev-roms/anguna.gba").expect("Failed to read ROM");
    gba_emu::emulator::init();
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    gba_emu::emulator::load_rom(rom_data.len());
    
    // Run 3 frames (to get to the halted state)
    for _ in 0..3 {
        gba_emu::emulator::run_frame();
    }
    
    let emu = gba_emu::emulator::get_emu();
    let iwram: &[u8] = &emu.mem.iwram[..];
    
    // Check IRQ handler pointer
    let irq_handler = rd32(iwram, 0x7FFC);
    eprintln!("IRQ handler ptr: 0x{:08X}", irq_handler);
    
    // Check bios_if
    let bios_if = rd16(iwram, 0x7FF8);
    eprintln!("bios_if (IWRAM[0x7FF8]): 0x{:04X}", bios_if);
    
    // Dump IRQ handler code at 0x03000EF8
    let handler_off = (irq_handler & 0x7FFF) as usize;
    eprintln!("\nIRQ handler at IWRAM[0x{:04X}]:", handler_off);
    for i in 0..20 {
        let addr = handler_off + i * 2;
        if addr + 1 < iwram.len() {
            let instr = rd16(iwram, addr);
            eprintln!("  0x{:08X}: {:04X}  (THUMB)", 0x03000000 + addr, instr);
        }
    }
    
    // Also check: what's at IWRAM[0x7FFA] (SoftReset flag)
    let softreset_flag = iwram[0x7FFA];
    eprintln!("\nSoftReset flag (IWRAM[0x7FFA]): 0x{:02X}", softreset_flag);
    
    // Check IE, IF, IME
    let io: &[u8] = &emu.mem.io[..];
    eprintln!("IE=0x{:04X} IF=0x{:04X} IME=0x{:04X}", rd16(io, 0x200), rd16(io, 0x202), rd16(io, 0x208));
    eprintln!("DISPCNT=0x{:04X}", rd16(io, 0));
    eprintln!("CPU: halted={} vbwait={} pc=0x{:08X} cpsr=0x{:08X}", 
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cpu.r[15], emu.cpu.cpsr);
    
    // Now run one more frame and check what happens
    eprintln!("\n--- Running frame 4 ---");
    gba_emu::emulator::run_frame();
    
    let emu = gba_emu::emulator::get_emu();
    let io: &[u8] = &emu.mem.io[..];
    let iwram: &[u8] = &emu.mem.iwram[..];
    eprintln!("After frame 4:");
    eprintln!("IE=0x{:04X} IF=0x{:04X} IME=0x{:04X}", rd16(io, 0x200), rd16(io, 0x202), rd16(io, 0x208));
    eprintln!("DISPCNT=0x{:04X}", rd16(io, 0));
    eprintln!("CPU: halted={} vbwait={} pc=0x{:08X} cpsr=0x{:08X}", 
        emu.cpu.halted, emu.cpu.vblank_intr_wait, emu.cpu.r[15], emu.cpu.cpsr);
    eprintln!("IRQ handler ptr: 0x{:08X}", rd32(iwram, 0x7FFC));
    eprintln!("SoftReset flag: 0x{:02X}", iwram[0x7FFA]);
    
    // Check OAM
    let oam: &[u8] = &emu.mem.oam[..];
    let mut oam_nz = 0;
    for b in oam.iter() { if *b != 0 { oam_nz += 1; } }
    eprintln!("OAM non-zero bytes: {}/1024", oam_nz);
    
    // Check VRAM
    let vram: &[u8] = &emu.mem.vram[..];
    let mut vram_nz = 0;
    for b in 0..0x10000 {
        if vram[vram.len() - 1 - b] != 0 { vram_nz += 1; }
    }
    eprintln!("VRAM non-zero bytes in last 64KB: {}", vram_nz);
}
