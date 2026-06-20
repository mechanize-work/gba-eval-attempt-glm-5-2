// Timers
use crate::interrupt::{Interrupt, IRQ_TIMER0, IRQ_TIMER1, IRQ_TIMER2, IRQ_TIMER3};

pub struct Timer {
    pub cnt: [u16; 4],    // Control registers
    pub data: [u16; 4],   // Reload values
    pub counter: [u32; 4], // Current counter
    pub overflow_at: [u64; 4], // Cycle when timer overflows (for cascade timing)
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            cnt: [0; 4],
            data: [0; 4],
            counter: [0; 4],
            overflow_at: [0; 4],
        }
    }

    pub fn reset(&mut self) {
        self.cnt = [0; 4];
        self.data = [0; 4];
        self.counter = [0; 4];
        self.overflow_at = [0; 4];
    }

    // Prescaler values
    fn prescaler_shift(cnt: u16) -> u32 {
        match cnt & 0x3 {
            0 => 0,
            1 => 6,
            2 => 8,
            3 => 10,
            _ => 0,
        }
    }

    // Run timers for a given number of cycles
    pub fn run(&mut self, cycles: u32, irq: &mut Interrupt) {
        for i in 0..4 {
            let cnt = self.cnt[i];
            if cnt & 0x80 == 0 {
                continue; // Not enabled
            }

            let cascade = cnt & 4 != 0;
            if cascade && i > 0 {
                // Cascade: only increments on overflow of previous timer
                continue;
            }

            let shift = Self::prescaler_shift(cnt);
            self.counter[i] = self.counter[i].wrapping_add(cycles >> shift);

            // Check overflow
            if self.counter[i] >= 0x10000 {
                let overflows = self.counter[i] >> 16;
                // Reload from data value, preserving remainder
                let reload = self.data[i] as u32;
                self.counter[i] = reload + (self.counter[i] & 0xFFFF);

                // Reload
                let _reload = self.data[i] as u32;

                // Trigger IRQ
                if cnt & 0x40 != 0 {
                    match i {
                        0 => irq.signal(IRQ_TIMER0),
                        1 => irq.signal(IRQ_TIMER1),
                        2 => irq.signal(IRQ_TIMER2),
                        3 => irq.signal(IRQ_TIMER3),
                        _ => {}
                    }
                }

                // Cascade next timer
                if i < 3 {
                    let next_cascade = self.cnt[i + 1] & 4 != 0;
                    if next_cascade && (self.cnt[i + 1] & 0x80) != 0 {
                        self.counter[i + 1] = self.counter[i + 1].wrapping_add(overflows);
                        if self.counter[i + 1] >= 0x10000 {
                            let reload_next = self.data[i + 1] as u32;
                            self.counter[i + 1] = reload_next + (self.counter[i + 1] & 0xFFFF);
                            if self.cnt[i + 1] & 0x40 != 0 {
                                match i + 1 {
                                    1 => irq.signal(IRQ_TIMER1),
                                    2 => irq.signal(IRQ_TIMER2),
                                    3 => irq.signal(IRQ_TIMER3),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Read timer counter value
    pub fn read_counter(&self, i: usize) -> u16 {
        (self.counter[i] & 0xFFFF) as u16
    }

    // Write reload value
    pub fn write_data(&mut self, i: usize, val: u16) {
        self.data[i] = val;
    }

    // Write control register
    pub fn write_cnt(&mut self, i: usize, val: u16) {
        let old = self.cnt[i];
        let enabled = val & 0x80 != 0;
        let was_enabled = old & 0x80 != 0;

        if enabled && !was_enabled {
            // Timer just enabled - reload counter
            self.counter[i] = self.data[i] as u32;
        }

        self.cnt[i] = val;
    }
}
