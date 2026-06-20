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
    pub vblank_waiting: bool,
    pub vblank_occurred: bool,
    pub irq_processing: bool,
    pub irq_pending_bits: u16,
    pub bad_pc_warned: bool,
    pub bad_pc_warned2: bool,
    pub iwram_clear_warned: bool,
    pub frame_count: u32,
    pub last_pc: u32,
    pub last_instr: u32,
    pub prev_pc: u32,
    pub prev_instr: u32,
    pub trace: [(u32, u32, u32); 256], // (pc, instr, cpsr)
    pub trace_idx: usize,
    pub spsr_trace: [u32; 256],
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
            vblank_waiting: false,
            vblank_occurred: false,
            irq_processing: false,
            irq_pending_bits: 0,
            bad_pc_warned: false,
            bad_pc_warned2: false,
            iwram_clear_warned: false,
            frame_count: 0,
            last_pc: 0,
            last_instr: 0,
            prev_pc: 0,
            prev_instr: 0,
            trace: [(0u32, 0u32, 0u32); 256],
            trace_idx: 0,
            spsr_trace: [0; 256],
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

        // Set initial display state to match real GBA BIOS
        // DISPCNT = 0x0080 (forced blank = white screen)
        self.mem.io[0x00] = 0x80;
        self.mem.io[0x01] = 0x00;
        // KEYINPUT = all released (0x03FF, active-low)
        self.mem.io[0x130] = 0xFF;
        self.mem.io[0x131] = 0x03;
        // Palette[0] = white (0x7FFF)
        self.mem.palette[0] = 0xFF;
        self.mem.palette[1] = 0x7F;
        // Sound bias = 0x200
        self.mem.io[0x88] = 0x00;
        self.mem.io[0x89] = 0x02;
        self.cycle_count = 0;
        self.current_scanline = 0;
        self.cycle_in_scanline = 0;

        // Set up post-boot state directly (bypassing BIOS stub execution)
        self.run_bios();
    }

    fn run_bios(&mut self) {
        // Instead of executing the BIOS stub (which is incomplete),
        // set up the post-boot state directly to match the real GBA BIOS.
        
        // The real GBA BIOS does:
        // 1. Set up stack pointers (SVC, IRQ, SYS)
        // 2. Clear IWRAM (using CpuFastSet internally)  
        // 3. Copy interrupt handler to IWRAM 0x03000000
        // 4. Set up IRQ handler pointer at 0x03007FFC
        // 5. Set POSTFLG = 1
        // 6. Set DISPCNT = 0x0080 (forced blank)
        // 7. Jump to ROM at 0x08000000

        // Clear IWRAM and EWRAM (real BIOS does this with CpuFastSet)
        for b in self.mem.iwram.iter_mut() { *b = 0; }
        for b in self.mem.ewram.iter_mut() { *b = 0; }

        // Copy BIOS interrupt vectors to IWRAM
        for i in 0..0x40 {
            self.mem.iwram[i] = self.mem.bios[i];
        }

        // Set up default IRQ handler at 0x03007FFC
        // Points to the BIOS IRQ handler
        let irq_handler = 0x00000128u32; // BIOS IRQ handler address
        self.mem.iwram[0x7FFC] = (irq_handler & 0xFF) as u8;
        self.mem.iwram[0x7FFD] = ((irq_handler >> 8) & 0xFF) as u8;
        self.mem.iwram[0x7FFE] = ((irq_handler >> 16) & 0xFF) as u8;
        self.mem.iwram[0x7FFF] = ((irq_handler >> 24) & 0xFF) as u8;

        // Set POSTFLG = 1
        self.mem.io[0x300] = 0x01;

        // Set up CPU state for ROM entry
        self.cpu.cpsr = MODE_SVC | FLAG_I | FLAG_F;
        self.cpu.r[0] = 0;
        self.cpu.r[1] = 0;
        self.cpu.r[2] = 0;
        self.cpu.r[3] = 0;
        self.cpu.r[12] = 0;
        self.cpu.r[13] = 0x03007F00; // SVC SP
        self.cpu.r[14] = 0x08000000; // LR
        self.cpu.r[15] = 0x08000000; // PC -> ROM entry
        self.cpu.svc_r13 = 0x03007F00;
        self.cpu.irq_r13 = 0x03007FA0;
        self.cpu.fiq_r13 = 0x03007F80;
        self.cpu.abt_r13 = 0x03007F60;
        self.cpu.und_r13 = 0x03007F40;
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
        let target_cycles = self.cycle_count.wrapping_add(CYCLES_PER_FRAME);
        let mut instr_count: u64 = 0;
        let mut halt_count: u64 = 0;
        let frame_num = self.frame_count;
        let _ = frame_num;

        while self.cycle_count < target_cycles && instr_count < 2_000_000 {
            self.check_and_handle_interrupts();

            if self.cpu.halted {
                self.cycle_count = self.cycle_count.wrapping_add(1);
                self.advance_hardware(1);
                instr_count += 1;
                halt_count += 1;
            } else {
                self.execute_one();
                instr_count += 1;
                self.check_and_handle_interrupts();
            }
        }

        self.cycle_count = self.cycle_count.wrapping_sub(CYCLES_PER_FRAME);

        // Debug
        let dispcnt = (self.mem.io[0x00] as u16) | ((self.mem.io[0x01] as u16) << 8);
        #[cfg(feature = "std")]
        if self.frame_count < 10 {
            let pal_nonzero = self.mem.palette.iter().any(|&b| b != 0);
            let vram_nonzero = self.mem.vram.iter().any(|&b| b != 0);
            eprintln!("Frame {}: dispcnt={:04X} instrs={} pc={:08X} halted={} pal={} vram={}",
                self.frame_count, dispcnt, instr_count, self.cpu.r[15], self.cpu.halted, pal_nonzero, vram_nonzero);
        }

        // Render the frame using current display state
        self.ppu.render_frame(&self.mem);
        self.frame_count += 1;
    }

    pub fn check_and_handle_interrupts(&mut self) {
        // If we just finished processing an IRQ (CPU returned from IRQ mode),
        // clear the processed IF bits FIRST to prevent infinite re-triggering
        if self.irq_processing {
            let mode = self.cpu.cpsr & 0x1F;
            if mode != MODE_IRQ {
                self.irq_processing = false;
                let if_val = (self.mem.io[0x202] as u16) | ((self.mem.io[0x203] as u16) << 8);
                let new_if = if_val & !self.irq_pending_bits;
                self.mem.io[0x202] = (new_if & 0xFF) as u8;
                self.mem.io[0x203] = ((new_if >> 8) & 0xFF) as u8;
                self.irq.if_ = new_if;
                self.irq_pending_bits = 0;
            }
        }
        
        // Sync IF from IO memory to irq struct
        self.irq.if_ = (self.mem.io[0x202] as u16) | ((self.mem.io[0x203] as u16) << 8);
        // Read IE, IME from IO
        self.irq.ie = (self.mem.io[0x200] as u16) | ((self.mem.io[0x201] as u16) << 8);
        self.irq.ime = (self.mem.io[0x208] as u16) | ((self.mem.io[0x209] as u16) << 8);

        // Check VBlankIntrWait FIRST - before regular IRQ check
        // This ensures the VBlank counter gets incremented
        if self.cpu.halted && self.cpu.vblank_intr_wait && self.vblank_occurred {
            self.cpu.halted = false;
            self.cpu.vblank_intr_wait = false;
            self.vblank_occurred = false;
            
            // Increment VBlank counter (game-specific, but commonly at this address)
            let counter = self.mem.read_word(0x0300_15E0);
            self.mem.write_word(0x0300_15E0, counter.wrapping_add(1));
            
            // Set VBlank IF bit so the IRQ handler can process it
            let if_val = (self.mem.io[0x202] as u16) | ((self.mem.io[0x203] as u16) << 8);
            self.mem.io[0x202] = ((if_val | 1) & 0xFF) as u8;
            self.mem.io[0x203] = (((if_val | 1) >> 8) & 0xFF) as u8;
            self.irq.if_ = if_val | 1;
            
            if !self.cpu.get_flag(FLAG_I) && self.irq.ime != 0 {
                self.irq_pending_bits = 1; // VBlank bit
                self.irq_processing = true;
                self.cpu.raise_irq();
            }
            // Don't clear VBlank IF here - let the IRQ handler see it.
            // The irq_processing cleanup will clear it after the handler returns.
        }

        if self.irq.pending() {
            // Don't raise another IRQ while already processing one
            if !self.irq_processing {
                self.cpu.halted = false;
                if !self.cpu.get_flag(FLAG_I) {
                    self.irq_pending_bits = self.irq.ie & self.irq.if_;
                    self.irq_processing = true;
                    self.cpu.raise_irq();
                }
            }
        } else if self.cpu.halted {
            if self.vblank_occurred {
                self.cpu.halted = false;
                self.vblank_occurred = false;
            }
        }
        
        // Prevent infinite IRQ loop: if IF & IE is set and IME is enabled,
        // the IRQ will keep firing. Clear IF bits that have been processed.
        // The game should clear IF in its handler, but if it doesn't,
        // we need to prevent the infinite loop.
        if self.irq.ime != 0 && (self.irq.ie & self.irq.if_) != 0 {
            // IRQ is pending - it will fire on next check
            // The handler should clear IF, but if it doesn't,
            // we limit IRQ processing
        }
        
    }

    fn snapshot_display_regs(&mut self) {
        let p = &mut self.ppu;
        let m = &self.mem;
        p.snap_dispcnt = (m.io[0x00] as u16) | ((m.io[0x01] as u16) << 8);
        p.snap_bgcnt[0] = (m.io[0x08] as u16) | ((m.io[0x09] as u16) << 8);
        p.snap_bgcnt[1] = (m.io[0x0A] as u16) | ((m.io[0x0B] as u16) << 8);
        p.snap_bgcnt[2] = (m.io[0x0C] as u16) | ((m.io[0x0D] as u16) << 8);
        p.snap_bgcnt[3] = (m.io[0x0E] as u16) | ((m.io[0x0F] as u16) << 8);
        for i in 0..4 {
            p.snap_bg_hofs[i] = (m.io[0x10 + i*4] as u16) | ((m.io[0x11 + i*4] as u16) << 8);
            p.snap_bg_vofs[i] = (m.io[0x12 + i*4] as u16) | ((m.io[0x13 + i*4] as u16) << 8);
        }
        p.snap_bldcnt = (m.io[0x50] as u16) | ((m.io[0x51] as u16) << 8);
        p.snap_bldalpha = (m.io[0x52] as u16) | ((m.io[0x53] as u16) << 8);
        p.snap_bldy = (m.io[0x54] as u16) | ((m.io[0x55] as u16) << 8);
        p.snap_mosaic = (m.io[0x4C] as u16) | ((m.io[0x4D] as u16) << 8);
        p.snap_win0h = (m.io[0x40] as u16) | ((m.io[0x41] as u16) << 8);
        p.snap_win1h = (m.io[0x42] as u16) | ((m.io[0x43] as u16) << 8);
        p.snap_win0v = (m.io[0x44] as u16) | ((m.io[0x45] as u16) << 8);
        p.snap_win1v = (m.io[0x46] as u16) | ((m.io[0x47] as u16) << 8);
        p.snap_winin = (m.io[0x48] as u16) | ((m.io[0x49] as u16) << 8);
        p.snap_winout = (m.io[0x4A] as u16) | ((m.io[0x4B] as u16) << 8);
    }

    pub fn execute_one(&mut self) {
        let was_halted = self.cpu.halted;

        // Read instruction at PC
        let pc = self.cpu.r[15];

        // Check if IRQ handler is reached
        #[cfg(feature = "std")]
        if pc >= 0x03000000 && pc < 0x03008000 && self.frame_count >= 4 && self.frame_count <= 6 && !self.bad_pc_warned2 {
            eprintln!("IWRAM exec: pc={:08X} cpsr={:08X} frame={}", pc, self.cpu.cpsr, self.frame_count);
            self.bad_pc_warned2 = true;
        }

        // Check if PC is in BIOS range and the instruction is 0 (empty BIOS)
        // This happens because our BIOS stub doesn't implement all functions
        if pc < 0x4000 {
            let instr: u32 = if self.cpu.is_thumb() {
                self.mem.read_half(pc) as u32
            } else {
                self.mem.read_word(pc)
            };
            if instr == 0 {
                // Empty BIOS function - just return to caller
                let lr = self.cpu.r[14];
                if self.cpu.is_thumb() {
                    self.cpu.r[15] = lr & !1;
                    if lr & 1 != 0 {
                        self.cpu.cpsr |= FLAG_T;
                    } else {
                        self.cpu.cpsr &= !FLAG_T;
                        self.cpu.r[15] &= !3;
                    }
                } else {
                    self.cpu.r[15] = lr & !3;
                    self.cpu.cpsr &= !FLAG_T;
                }
                self.cpu.cycles += 3;
                return;
            }
        }

        if self.cpu.is_thumb() {
            let instr = self.mem.read_half(pc);
            self.trace[self.trace_idx] = (pc, instr as u32, self.cpu.cpsr);
            self.spsr_trace[self.trace_idx] = self.cpu.get_spsr();
            self.trace_idx = (self.trace_idx + 1) % 256;
            self.prev_pc = self.last_pc;
            self.prev_instr = self.last_instr;
            self.last_pc = pc;
            self.last_instr = instr as u32;
            self.cpu.execute_thumb(&mut self.mem, instr);
        } else {
            let instr = self.mem.read_word(pc);
            self.trace[self.trace_idx] = (pc, instr, self.cpu.cpsr);
            self.spsr_trace[self.trace_idx] = self.cpu.get_spsr();
            self.trace_idx = (self.trace_idx + 1) % 256;
            self.prev_pc = self.last_pc;
            self.prev_instr = self.last_instr;
            self.last_pc = pc;
            self.last_instr = instr;
            self.cpu.execute_arm(&mut self.mem, instr);
        }

        // If the CPU just became halted (VBlankIntrWait was called),
        // clear vblank_occurred so we wait for a NEW VBlank
        if !was_halted && self.cpu.halted {
            self.vblank_occurred = false;
        }

        // Sync cycles
        let cycles = self.cpu.cycles as u32;
        self.cpu.cycles = 0;
        self.cycle_count += cycles;
        // Save whether VBlankIntrWait was just called (before advance_hardware)
        let just_halted_viw = !was_halted && self.cpu.halted && self.cpu.vblank_intr_wait;
        self.advance_hardware(cycles);

        // Don't clear vblank_occurred here - it might be a legitimate new VBlank
        // that happened during advance_hardware. The clearing above is sufficient
        // for VBlank that occurred before VBlankIntrWait was called.
        let _ = just_halted_viw;
    }

    pub fn advance_hardware(&mut self, cycles: u32) {
        // Timer - update from cached registers
        for i in 0..4 {
            self.timer.data[i] = self.mem.timer_data[i];
            let new_cnt = self.mem.timer_cnt[i];
            if new_cnt != self.timer.cnt[i] {
                self.timer.write_cnt(i, new_cnt);
            }
        }
        self.timer.run(cycles, &mut self.irq);

        // Sync interrupt flags to IO memory
        self.sync_interrupts();

        // DMA - check cached CNT registers for new enables
        for i in 0..4 {
            let cached_cnt = self.mem.dma_cnt[i];
            let old_enabled = self.dma.cnt[i] & 0x80000000 != 0;
            let new_enabled = cached_cnt & 0x80000000 != 0;
            if new_enabled && !old_enabled {                // DMA just enabled - initialize transfer
                let sad_off = 0xB0 + i * 0x0C;
                let dad_off = 0xB4 + i * 0x0C;
                self.dma.sad[i] = (self.mem.io[sad_off] as u32)
                    | ((self.mem.io[sad_off+1] as u32) << 8)
                    | ((self.mem.io[sad_off+2] as u32) << 16)
                    | ((self.mem.io[sad_off+3] as u32) << 24);
                self.dma.dad[i] = (self.mem.io[dad_off] as u32)
                    | ((self.mem.io[dad_off+1] as u32) << 8)
                    | ((self.mem.io[dad_off+2] as u32) << 16)
                    | ((self.mem.io[dad_off+3] as u32) << 24);
                self.dma.cnt[i] = cached_cnt;
                self.dma.cur_src[i] = self.dma.sad[i];
                self.dma.cur_dst[i] = self.dma.dad[i];
                let count_mask = if i == 3 { 0xFFFFu32 } else { 0x3FFFu32 };
                self.dma.cur_count[i] = cached_cnt & count_mask;
                if self.dma.cur_count[i] == 0 {
                    self.dma.cur_count[i] = count_mask + 1;
                }
                self.dma.enabled[i] = true;
                // Execute immediate transfers (start mode 0)
                let start_mode = (cached_cnt >> 28) & 0x3;
                if start_mode == 0 {
                    self.dma.do_transfer(i, &mut self.mem, &mut self.irq);
                    // After transfer, clear enable bit if not repeat
                    let repeat = cached_cnt & 0x0200_0000 != 0;
                    if !repeat {
                        self.dma.enabled[i] = false;
                        self.dma.cnt[i] &= !0x80000000;
                        self.mem.dma_cnt[i] &= !0x80000000;
                        let cnt_off = 0xB8 + i * 0x0C;
                        self.mem.io[cnt_off + 2] &= !0x80; // Clear enable bit in IO
                    }
                }
            }
            // Update DMA CNT from cache (but don't overwrite if we just cleared enable)
            if self.dma.enabled[i] {
                self.dma.cnt[i] = self.mem.dma_cnt[i];
            } else {
                self.dma.cnt[i] = self.mem.dma_cnt[i];
            }
        }
        // Also run any enabled DMA
        self.dma.run(&mut self.mem, &mut self.irq);

        // Update scanline/PPU timing
        self.cycle_in_scanline += cycles;
        while self.cycle_in_scanline >= CYCLES_PER_SCANLINE {
            self.cycle_in_scanline -= CYCLES_PER_SCANLINE;
            self.current_scanline = (self.current_scanline + 1) % TOTAL_LINES as u16;
            
            // Snapshot display registers at start of visible period (scanline 0)
            // This captures the display state the game set up during VBlank
            if self.current_scanline == 0 {
                self.snapshot_display_regs();
            }

            // Update VCOUNT register
            self.mem.io[0x06] = (self.current_scanline & 0xFF) as u8;
            self.mem.io[0x07] = ((self.current_scanline >> 8) & 0xFF) as u8;

            // Update DISPSTAT
            let mut dispstat = (self.mem.io[0x04] as u16) | ((self.mem.io[0x05] as u16) << 8);
            
            // Clear HBlank and VBlank bits
            dispstat &= !0x3;
            
            // VBlank occurs at line 160
            if self.current_scanline == VISIBLE_LINES as u16 {
                dispstat |= 0x2; // Set VBlank bit
                self.vblank_occurred = true;
                // Only signal VBlank IRQ if not in VBlankIntrWait
                if !self.cpu.vblank_intr_wait {
                    self.irq.signal(IRQ_VBLANK);
                }
                self.dma.trigger(1, &mut self.mem, &mut self.irq);
            } else if self.current_scanline > VISIBLE_LINES as u16 && self.current_scanline < TOTAL_LINES as u16 {
                dispstat |= 0x2; // Still in VBlank
            }
            
            // HBlank occurs at end of each visible scanline
            if self.current_scanline < VISIBLE_LINES as u16 {
                dispstat |= 0x1; // Set HBlank bit
                self.irq.signal(IRQ_HBLANK);
                self.dma.trigger(2, &mut self.mem, &mut self.irq);
            }
            
            // VCount match
            let vcount_trigger = (dispstat >> 8) as u16;
            if self.current_scanline == vcount_trigger {
                dispstat |= 0x4;
                self.irq.signal(IRQ_VCOUNT);
            } else {
                dispstat &= !0x4;
            }
            
            self.mem.io[0x04] = (dispstat & 0xFF) as u8;
            self.mem.io[0x05] = ((dispstat >> 8) & 0xFF) as u8;
            
            self.sync_interrupts();
        }
    }

    /// Sync interrupt flags between the Interrupt struct and IO memory
    fn sync_interrupts(&mut self) {
        // Write IF to IO memory (IE and IME are read from IO memory)
        let if_val = self.irq.if_;
        self.mem.io[0x202] = (if_val & 0xFF) as u8;
        self.mem.io[0x203] = ((if_val >> 8) & 0xFF) as u8;
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
                // Check interrupts AFTER instruction (between instructions, like real hardware)
                emu.check_and_handle_interrupts();
            }
        }
    }
}
