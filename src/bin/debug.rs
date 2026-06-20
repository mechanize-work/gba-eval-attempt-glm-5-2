// Debug harness - runs emulator with debug output
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_file = if args.len() > 1 { &args[1] } else { "dev-roms/anguna.gba" };
    let max_steps: u32 = if args.len() > 2 { args[2].parse().unwrap() } else { 100 };

    let rom_data = fs::read(rom_file).expect("Failed to read ROM");
    eprintln!("ROM size: {} bytes", rom_data.len());

    gba_emu::emulator::init();
    
    let rom_ptr = gba_emu::emulator::rom_buffer_ptr();
    unsafe {
        let rom_slice = std::slice::from_raw_parts_mut(rom_ptr, rom_data.len());
        rom_slice.copy_from_slice(&rom_data);
    }
    
    let result = gba_emu::emulator::load_rom(rom_data.len());
    eprintln!("emu_load_rom returned: {}", result);

    // Get debug info from the emulator
    let emu = gba_emu::emulator::get_emu();
    
    eprintln!("\nAfter BIOS execution:");
    eprintln!("  PC: 0x{:08X}", emu.cpu.r[15]);
    eprintln!("  CPSR: 0x{:08X}", emu.cpu.cpsr);
    eprintln!("  Mode: 0x{:02X}", emu.cpu.cpsr & 0x1F);
    eprintln!("  THUMB: {}", emu.cpu.is_thumb());
    eprintln!("  R0: 0x{:08X}", emu.cpu.r[0]);
    eprintln!("  R1: 0x{:08X}", emu.cpu.r[1]);
    eprintln!("  R12: 0x{:08X}", emu.cpu.r[12]);
    eprintln!("  R13(SP): 0x{:08X}", emu.cpu.r[13]);
    eprintln!("  R14(LR): 0x{:08X}", emu.cpu.r[14]);
    
    // Check what's at the ROM entry point
    let entry = emu.cpu.r[15];
    eprintln!("\n  Entry point: 0x{:08X}", entry);
    if entry >= 0x0800_0000 {
        let rom_offset = (entry - 0x0800_0000) as usize;
        eprintln!("  ROM offset: 0x{:08X}", rom_offset);
        if rom_offset < emu.mem.rom_size {
            let instr = emu.mem.read_rom_word(entry);
            eprintln!("  First instruction: 0x{:08X}", instr);
            // Check if it's a branch
            if (instr >> 24) == 0xEA {
                let offset = (instr & 0x00FF_FFFF) as i32;
                let offset = if offset & 0x0080_0000 != 0 { offset | (0xFF00_0000u32 as i32) } else { offset };
                let target = entry.wrapping_add(8).wrapping_add((offset as u32).wrapping_mul(4));
                eprintln!("  Branch target: 0x{:08X}", target);
            }
        }
    }
    
    // Check the ROM header
    eprintln!("\nROM header:");
    eprintln!("  Entry: 0x{:08X}", emu.mem.read_rom_word(0x0800_0000));
    eprintln!("  Nintendo logo (first 4 bytes): 0x{:08X}", emu.mem.read_rom_word(0x0800_0004));
    eprintln!("  Title: {:?}", std::str::from_utf8(&emu.mem.rom[0xA0..0xAC]).unwrap_or("(invalid)"));
    eprintln!("  Game code: {:?}", std::str::from_utf8(&emu.mem.rom[0xAC..0xB0]).unwrap_or("(invalid)"));
    
    // Run a few instructions manually and trace
    eprintln!("\nTracing first {} instructions:", max_steps);
    for i in 0..max_steps {
        let pc = emu.cpu.r[15];
        let thumb = emu.cpu.is_thumb();
        
        if thumb {
            let instr = emu.mem.read_half(pc);
            eprintln!("[{:4}] PC=0x{:08X} THUMB 0x{:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} R8={:08X} R9={:08X} R10={:08X} R11={:08X} R12={:08X} SP={:08X} LR={:08X} CPSR={:08X}",
                i, pc, instr,
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7],
                emu.cpu.r[8], emu.cpu.r[9], emu.cpu.r[10], emu.cpu.r[11],
                emu.cpu.r[12], emu.cpu.r[13], emu.cpu.r[14], emu.cpu.cpsr);
        } else {
            let instr = emu.mem.read_word(pc);
            eprintln!("[{:4}] PC=0x{:08X} ARM 0x{:08X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} R8={:08X} R9={:08X} R10={:08X} R11={:08X} R12={:08X} SP={:08X} LR={:08X} CPSR={:08X}",
                i, pc, instr,
                emu.cpu.r[0], emu.cpu.r[1], emu.cpu.r[2], emu.cpu.r[3],
                emu.cpu.r[4], emu.cpu.r[5], emu.cpu.r[6], emu.cpu.r[7],
                emu.cpu.r[8], emu.cpu.r[9], emu.cpu.r[10], emu.cpu.r[11],
                emu.cpu.r[12], emu.cpu.r[13], emu.cpu.r[14], emu.cpu.cpsr);
        }
        
        gba_emu::emulator::step_one();
        
        if emu.cpu.halted {
            eprintln!("CPU halted!");
            break;
        }
    }
    
    // Check DISPCNT
    let dispcnt = (emu.mem.io[0x00] as u16) | ((emu.mem.io[0x01] as u16) << 8);
    eprintln!("\nDISPCNT: 0x{:04X} (mode={}, bg_en=0x{:X})", dispcnt, dispcnt & 7, (dispcnt >> 8) & 0xF);
    let vcount = (emu.mem.io[0x06] as u16) | ((emu.mem.io[0x07] as u16) << 8);
    eprintln!("VCOUNT: {}", vcount);
    let ime = (emu.mem.io[0x208] as u16) | ((emu.mem.io[0x209] as u16) << 8);
    let ie = (emu.mem.io[0x200] as u16) | ((emu.mem.io[0x201] as u16) << 8);
    let if_ = (emu.mem.io[0x202] as u16) | ((emu.mem.io[0x203] as u16) << 8);
    eprintln!("IME: {} IE: 0x{:04X} IF: 0x{:04X}", ime, ie, if_);
}
