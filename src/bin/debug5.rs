// Debug: find when DISPCNT changes or CPU halts
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
    
    let mut last_dispcnt: u16 = 0;
    let mut last_halted = false;
    
    for i in 0..10_000_000u32 {
        gba_emu::emulator::step_one();
        
        let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
        let halted = emu.cpu.halted;
        
        if dispcnt != last_dispcnt {
            eprintln!("[{}] DISPCNT changed: 0x{:04X} -> 0x{:04X} PC=0x{:08X} cycles={}",
                i, last_dispcnt, dispcnt, emu.cpu.r[15], emu.cycle_count);
            last_dispcnt = dispcnt;
        }
        
        if halted && !last_halted {
            eprintln!("[{}] CPU halted! PC=0x{:08X} cycles={}", i, emu.cpu.r[15], emu.cycle_count);
            let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
            let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
            let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
            let dispstat = (emu.mem.io[0x04] as u16) | ((emu.mem.io[0x05] as u16) << 8);
            let vcount = (emu.mem.io[0x06] as u16) | ((emu.mem.io[0x07] as u16) << 8);
            eprintln!("  IME={} IE=0x{:04X} IF=0x{:04X} DISPSTAT=0x{:04X} VCOUNT={}",
                ime, ie, if_, dispstat, vcount);
            last_halted = true;
            break;
        }
        
        if i % 1_000_000 == 0 && i > 0 {
            eprintln!("[{}] Still running... PC=0x{:08X} cycles={} DISPCNT=0x{:04X}",
                i, emu.cpu.r[15], emu.cycle_count, dispcnt);
        }
    }
}
