// DMA controller
use crate::memory::Memory;
use crate::interrupt::{Interrupt, IRQ_DMA0, IRQ_DMA1, IRQ_DMA2, IRQ_DMA3};

pub struct Dma {
    pub sad: [u32; 4],
    pub dad: [u32; 4],
    pub cnt: [u32; 4],  // Control registers (full 32-bit for DMA3, 16-bit for 0-2)
    pub enabled: [bool; 4],
    pub cur_src: [u32; 4],
    pub cur_dst: [u32; 4],
    pub cur_count: [u32; 4],
}

impl Dma {
    pub fn new() -> Self {
        Dma {
            sad: [0; 4],
            dad: [0; 4],
            cnt: [0; 4],
            enabled: [false; 4],
            cur_src: [0; 4],
            cur_dst: [0; 4],
            cur_count: [0; 4],
        }
    }

    pub fn reset(&mut self) {
        self.sad = [0; 4];
        self.dad = [0; 4];
        self.cnt = [0; 4];
        self.enabled = [false; 4];
        self.cur_src = [0; 4];
        self.cur_dst = [0; 4];
        self.cur_count = [0; 4];
    }

    fn src_mask(i: usize) -> u32 {
        match i {
            0 => 0x07FF_FFFF,
            1 | 2 => 0x0FFF_FFFF,
            3 => 0x0FFF_FFFF,
            _ => 0x0FFF_FFFF,
        }
    }

    fn dst_mask(i: usize) -> u32 {
        match i {
            0 | 1 | 2 => 0x07FF_FFFF,
            3 => 0x0FFF_FFFF,
            _ => 0x0FFF_FFFF,
        }
    }

    fn count_mask(i: usize) -> u32 {
        match i {
            3 => 0xFFFF,
            _ => 0x3FFF,
        }
    }

    pub fn write_cnt(&mut self, i: usize, val: u32, irq: &mut Interrupt) {
        let old = self.cnt[i];
        self.cnt[i] = val;

        // Check enable bit
        let was_enabled = old & 0x8000_0000 != 0;
        let now_enabled = val & 0x8000_0000 != 0;

        if now_enabled && !was_enabled {
            // DMA just enabled - initialize transfer
            self.cur_src[i] = self.sad[i] & Self::src_mask(i);
            self.cur_dst[i] = self.dad[i] & Self::dst_mask(i);
            self.cur_count[i] = (val & Self::count_mask(i)) as u32;
            if self.cur_count[i] == 0 {
                self.cur_count[i] = Self::count_mask(i) + 1;
            }
            self.enabled[i] = true;

            // Handle immediate transfer (start mode 0 = immediately)
            let start_mode = (val >> 12) & 0x3;
            if start_mode == 0 {
                // Will be processed in run()
            }
        }
    }

    // Process DMA transfers, returns true if any DMA was active
    pub fn run(&mut self, mem: &mut Memory, irq: &mut Interrupt) -> bool {
        let mut any_active = false;

        for i in 0..4 {
            if !self.enabled[i] {
                continue;
            }

            let start_mode = (self.cnt[i] >> 12) & 0x3;
            // Only handle immediate transfers here
            // Special start modes (VBlank, HBlank, Special) are triggered externally
            if start_mode != 0 {
                continue;
            }

            any_active = true;
            self.do_transfer(i, mem, irq);
        }

        any_active
    }

    pub fn do_transfer(&mut self, i: usize, mem: &mut Memory, irq: &mut Interrupt) {
        let cnt = self.cnt[i];
        let is_32bit = cnt & 0x0400 != 0;
        let src_adj = (cnt >> 7) & 0x3;
        let dst_adj = (cnt >> 5) & 0x3;
        let repeat = cnt & 0x0200 != 0;
        let irq_en = cnt & 0x4000 != 0;

        let size = if is_32bit { 4 } else { 2 };
        let count = self.cur_count[i];

        for _ in 0..count {
            if is_32bit {
                let val = mem.read_word(self.cur_src[i]);
                mem.write_word(self.cur_dst[i], val);
            } else {
                let val = mem.read_half(self.cur_src[i]);
                mem.write_half(self.cur_dst[i], val);
            }

            // Source address adjustment
            match src_adj {
                0 => { self.cur_src[i] = self.cur_src[i].wrapping_add(size); }
                1 => { self.cur_src[i] = self.cur_src[i].wrapping_sub(size); }
                2 => {} // Fixed
                3 => {} // Prohibited / reload
                _ => {}
            }

            // Dest address adjustment
            match dst_adj {
                0 => { self.cur_dst[i] = self.cur_dst[i].wrapping_add(size); }
                1 => { self.cur_dst[i] = self.cur_dst[i].wrapping_sub(size); }
                2 => {} // Fixed
                3 => {} // Reload src on repeat
                _ => {}
            }
        }

        if !repeat {
            self.enabled[i] = false;
            // Clear enable bit
            self.cnt[i] &= !0x8000_0000;
        } else {
            // Reload src for prohibited src_adj=3
            // Reload count
            self.cur_count[i] = (self.cnt[i] & Self::count_mask(i)) as u32;
            if self.cur_count[i] == 0 {
                self.cur_count[i] = Self::count_mask(i) + 1;
            }
        }

        if irq_en {
            match i {
                0 => irq.signal(IRQ_DMA0),
                1 => irq.signal(IRQ_DMA1),
                2 => irq.signal(IRQ_DMA2),
                3 => irq.signal(IRQ_DMA3),
                _ => {}
            }
        }
    }

    // Trigger DMA for a specific start mode (1=VBlank, 2=HBlank, 3=Special)
    pub fn trigger(&mut self, start_mode: u32, mem: &mut Memory, irq: &mut Interrupt) {
        for i in 0..4 {
            if !self.enabled[i] {
                continue;
            }
            let sm = (self.cnt[i] >> 12) & 0x3;
            if sm == start_mode {
                self.do_transfer(i, mem, irq);
            }
        }
    }

    // Trigger special (FIFO) DMA for channels 1 or 2
    pub fn trigger_fifo(&mut self, channel: usize, mem: &mut Memory, irq: &mut Interrupt) {
        if !self.enabled[channel] {
            return;
        }
        let sm = (self.cnt[channel] >> 12) & 0x3;
        if sm == 3 {
            // FIFO transfer: 4 bytes from src to FIFO
            let cnt = self.cnt[channel];
            let src_adj = (cnt >> 7) & 0x3;

            let val = mem.read_word(self.cur_src[channel]);
            // Write to FIFO address
            let fifo_addr = if channel == 1 {
                0x0400_00A0
            } else {
                0x0400_00A4
            };
            mem.write_word(fifo_addr, val);

            match src_adj {
                0 => { self.cur_src[channel] = self.cur_src[channel].wrapping_add(4); }
                1 => { self.cur_src[channel] = self.cur_src[channel].wrapping_sub(4); }
                2 => {}
                3 => {}
                _ => {}
            }

            // Check if we need to repeat
            let repeat = cnt & 0x0200 != 0;
            let irq_en = cnt & 0x4000 != 0;

            if !repeat {
                self.enabled[channel] = false;
                self.cnt[channel] &= !0x8000_0000;
            }

            if irq_en {
                match channel {
                    1 => irq.signal(IRQ_DMA1),
                    2 => irq.signal(IRQ_DMA2),
                    _ => {}
                }
            }
        }
    }
}
