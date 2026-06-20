// Main emulator module - ties CPU, memory, PPU, APU, etc together
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::cpu::*;
use crate::memory::*;
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::dma::Dma;
use crate::timer::Timer;
use crate::input::Input;
use crate::interrupt::{Interrupt, IRQ_VBLANK, IRQ_HBLANK, IRQ_VCOUNT};

// GBA clock speed: 16.78 MHz
const CPU_CLOCK: u32 = 16_777_216;
// Cycles per frame: 280896 (4 scanline types)
const CYCLES_PER_FRAME: u32 = 280_896;
const CYCLES_PER_SCANLINE: u32 = 1232;
const VISIBLE_LINES: u32 = 160;
const TOTAL_LINES: u32 = 228;

static mut EMU: Option<Emulator> = None;

pub struct Emulator {
    pub cpu: Cpu,
    pub mem: Memory,
    pub ppu: Ppu,
    pub apu: Apu,
    pub dma: Dma,
    pub timer: Timer,
    pub input: Input,
    pub irq: Interrupt,
    pub rom_data: Box<[u8]>,  // 32 MiB buffer for ROM loading
    pub cycle_count: u32,
    pub current_scanline: u16,
    pub cycle_in_scanline: u32,
}

impl Emulator {
    pub fn new() -> Self {
        let rom_data = {
            let mut v = Vec::new();
            v.resize(ROM_MAX_SIZE, 0u8);
            v.into_boxed_slice()
        };
        Emulator {
            cpu: Cpu::new(),
            mem: Memory::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            dma: Dma::new(),
            timer: Timer::new(),
            input: Input::new(),
            irq: Interrupt::new(),
            rom_data,
            cycle_count: 0,
            current_scanline: 0,
            cycle_in_scanline: 0,
        }
    }

    pub fn load_bios(&mut self) {
        // BIOS stub is embedded
        #[cfg(not(feature = "std"))]
        let bios = include_bytes!("../spec/gba_bios_stub.bin");
        #[cfg(feature = "std")]
        let bios = std::include_bytes!("../spec/gba_bios_stub.bin");
        self.mem.load_bios(bios);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.mem.reset();
        self.ppu.reset();
        self.apu.reset();
        self.dma.reset();
        self.timer.reset();
        self.irq.reset();
        self.load_bios();
        self.cycle_count = 0;
        self.current_scanline = 0;
        self.cycle_in_scanline = 0;

        // Set up initial register values
        // Post-boot state: these are set by BIOS, but since we have a stub,
        // we need to set them up ourselves
        // After BIOS execution, the ROM entry point at 0x08000000 should be called
        // with r0=0, r1=0, r2=0, r3=0, r12=0, sp=0x03007F00, lr=0x08000000, pc=0x08000000
        // and CPSR = SVC mode, IRQ disabled

        // Actually, let the BIOS stub execute. The BIOS stub sets up exception vectors
        // and then branches to the ROM entry point.
        // The BIOS stub reset handler at 0x0000 sets up:
        // - SVC stack
        // - IRQ stack
        // - System stack
        // Then jumps to 0x08000000

        // But we need to make sure ROM is mapped. Let's set PC=0 and let it run.
        self.cpu.r[15] = 0x0000_0000;
        self.cpu.cpsr = MODE_SVC | FLAG_I | FLAG_F;

        // Run the BIOS stub to initialize
        // The BIOS will set up stacks and jump to ROM at 0x08000000
        self.run_bios();
    }

    fn run_bios(&mut self) {
        // Execute BIOS until PC reaches 0x08000000
        let max_bios_cycles = 1_000_000;
        for _ in 0..max_bios_cycles {
            if self.cpu.r[15] >= 0x0800_0000 {
                break;
            }
            self.execute_one();
        }
    }

    pub fn load_rom(&mut self, len: usize) -> i32 {
        if len > ROM_MAX_SIZE || len == 0 {
            return 0;
        }
        self.mem.load_rom(&self.rom_data[..len]);
        self.mem.rom_size = len;
        self.reset();
        1
    }

    pub fn set_keys(&mut self, keys: u32) {
        self.input.set_keys(keys);
        // Update KEYINPUT register
        let keyinput = self.input.keyinput();
        self.mem.io[0x130] = (keyinput & 0xFF) as u8;
        self.mem.io[0x131] = ((keyinput >> 8) & 0xFF) as u8;
        // Check keypad interrupt
        self.check_keypad_irq();
    }

    fn check_keypad_irq(&mut self) {
        let keycnt = ((self.mem.io[0x132] as u16) | ((self.mem.io[0x133] as u16) << 8)) as u16;
        if keycnt & 0x4000 == 0 {
            return; // IRQ not enabled
        }
        let keyinput = self.input.keyinput();
        let keys = !keyinput & 0x3FF;
        let mask = keycnt & 0x3FF;
        let irq = if keycnt & 0x8000 != 0 {
            // AND mode: all selected keys pressed
            (keys & mask) == mask
        } else {
            // OR mode: any selected key pressed
            (keys & mask) != 0
        };
        if irq {
            self.irq.signal(12); // Keypad IRQ
        }
    }

    pub fn run_frame(&mut self) {
        // Run CPU for one frame's worth of cycles
        let target_cycles = self.cycle_count + CYCLES_PER_FRAME;

        while self.cycle_count < target_cycles {
            // Check for interrupts
            self.check_and_handle_interrupts();

            // Execute one instruction
            if self.cpu.halted {
                // CPU is halted (HALTCNT), just advance cycles
                self.cycle_count += 1;
                self.advance_hardware(1);
            } else {
                self.execute_one();
            }
        }

        self.cycle_count = self.cycle_count.wrapping_sub(CYCLES_PER_FRAME);

        // Render the frame
        self.ppu.render_frame(&self.mem);

        // Generate audio
        self.apu.generate_frame(CYCLES_PER_FRAME);

        // Signal VBlank interrupt
        self.irq.signal(IRQ_VBLANK);
    }

    fn check_and_handle_interrupts(&mut self) {
        // Read IE, IF, IME from IO
        self.irq.ie = (self.mem.io[0x200] as u16) | ((self.mem.io[0x201] as u16) << 8);
        self.irq.if_ = (self.mem.io[0x202] as u16) | ((self.mem.io[0x203] as u16) << 8);
        self.irq.ime = (self.mem.io[0x208] as u16) | ((self.mem.io[0x209] as u16) << 8);

        if self.irq.pending() {
            self.cpu.halted = false;
            self.cpu.raise_irq();
        }
    }

    fn execute_one(&mut self) {
        // Read instruction at PC
        let pc = self.cpu.r[15];

        if self.cpu.is_thumb() {
            let instr = self.mem.read_half(pc);
            // Do NOT pre-increment PC here - the instruction handler does it
            self.cpu.execute_thumb(&mut self.mem, instr);
        } else {
            let instr = self.mem.read_word(pc);
            // Do NOT pre-increment PC here - the instruction handler does it
            self.cpu.execute_arm(&mut self.mem, instr);
        }

        // Sync cycles
        let cycles = self.cpu.cycles as u32;
        self.cpu.cycles = 0;
        self.cycle_count += cycles;
        self.advance_hardware(cycles);
    }

    fn advance_hardware(&mut self, cycles: u32) {
        // Timer
        self.timer.run(cycles, &mut self.irq);

        // DMA - check for immediate transfers
        self.dma.run(&mut self.mem, &mut self.irq);

        // APU - clock FIFO timers etc
        // (handled in generate_frame for now)

        // Update scanline/PPU timing
        self.cycle_in_scanline += cycles;
        while self.cycle_in_scanline >= CYCLES_PER_SCANLINE {
            self.cycle_in_scanline -= CYCLES_PER_SCANLINE;
            self.current_scanline = (self.current_scanline + 1) % TOTAL_LINES as u16;

            // Update VCOUNT
            self.mem.io[0x06] = (self.current_scanline & 0xFF) as u8;
            self.mem.io[0x07] = ((self.current_scanline >> 8) & 0xFF) as u8;

            // Check VBlank
            if self.current_scanline == VISIBLE_LINES as u16 {
                self.irq.signal(IRQ_VBLANK);
                // Trigger VBlank DMA
                self.dma.trigger(1, &mut self.mem, &mut self.irq);
            }

            // Check HBlank (start of HBlank = cycle 1006)
            // For simplicity, signal at end of scanline
            if self.current_scanline < VISIBLE_LINES as u16 {
                self.irq.signal(IRQ_HBLANK);
                self.dma.trigger(2, &mut self.mem, &mut self.irq);
            }

            // Check VCount match
            let dispstat = (self.mem.io[0x04] as u16) | ((self.mem.io[0x05] as u16) << 8);
            let vcount_trigger = (dispstat >> 8) as u16;
            if self.current_scanline == vcount_trigger {
                self.irq.signal(IRQ_VCOUNT);
            }
        }
    }

    pub fn framebuffer(&self) -> &[u32] {
        &self.ppu.framebuffer[..]
    }

    pub fn audio_buffer(&self) -> &[i16] {
        &self.apu.audio_buffer[..self.apu.audio_count * 2]
    }

    pub fn audio_samples(&mut self) -> usize {
        self.apu.drain_audio()
    }
}

// Global interface functions
pub fn init() {
    unsafe {
        if EMU.is_none() {
            let mut emu = Emulator::new();
            emu.load_bios();
            EMU = Some(emu);
        }
    }
}

pub fn rom_buffer_ptr() -> *mut u8 {
    unsafe {
        EMU.as_mut().unwrap().rom_data.as_mut_ptr()
    }
}

pub fn load_rom(len: usize) -> i32 {
    unsafe {
        if let Some(emu) = EMU.as_mut() {
            emu.load_rom(len)
        } else {
            0
        }
    }
}

pub fn reset() -> i32 {
    unsafe {
        if let Some(emu) = EMU.as_mut() {
            emu.reset();
            1
        } else {
            0
        }
    }
}

pub fn set_keys(keys: u32) {
    unsafe {
        if let Some(emu) = EMU.as_mut() {
            emu.set_keys(keys);
        }
    }
}

pub fn run_frame() {
    unsafe {
        if let Some(emu) = EMU.as_mut() {
            emu.run_frame();
        }
    }
}

pub fn framebuffer_ptr() -> *mut u32 {
    unsafe {
        EMU.as_mut().unwrap().ppu.framebuffer.as_mut_ptr()
    }
}

pub fn audio_buffer_ptr() -> *mut i16 {
    unsafe {
        EMU.as_mut().unwrap().apu.audio_buffer.as_mut_ptr()
    }
}

pub fn audio_samples() -> i32 {
    unsafe {
        EMU.as_mut().unwrap().audio_samples() as i32
    }
}

pub fn audio_rate() -> i32 {
    unsafe {
        EMU.as_ref().unwrap().apu.sample_rate as i32
    }
}

#[cfg(feature = "std")]
pub fn get_emu() -> &'static mut Emulator {
    unsafe {
        EMU.as_mut().unwrap()
    }
}

#[cfg(feature = "std")]
pub fn step_one() {
    unsafe {
        if let Some(emu) = EMU.as_mut() {
            emu.check_and_handle_interrupts();
            if emu.cpu.halted {
                emu.cycle_count += 1;
                emu.advance_hardware(1);
            } else {
                emu.execute_one();
            }
        }
    }
}
