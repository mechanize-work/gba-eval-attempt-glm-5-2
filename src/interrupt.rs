// Interrupt controller
pub struct Interrupt {
    pub ie: u16,   // Interrupt Enable
    pub if_: u16,  // Interrupt Flag (1 = pending)
    pub ime: u16,  // Interrupt Master Enable
}

impl Interrupt {
    pub fn new() -> Self {
        Interrupt {
            ie: 0,
            if_: 0,
            ime: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ie = 0;
        self.if_ = 0;
        self.ime = 0;
    }

    // Returns true if an interrupt should fire
    pub fn pending(&self) -> bool {
        self.ime != 0 && (self.ie & self.if_) != 0
    }

    // Signal an interrupt
    pub fn signal(&mut self, bit: u16) {
        self.if_ |= 1 << bit;
    }
}

// Interrupt bits
pub const IRQ_VBLANK: u16 = 0;
pub const IRQ_HBLANK: u16 = 1;
pub const IRQ_VCOUNT: u16 = 2;
pub const IRQ_TIMER0: u16 = 3;
pub const IRQ_TIMER1: u16 = 4;
pub const IRQ_TIMER2: u16 = 5;
pub const IRQ_TIMER3: u16 = 6;
pub const IRQ_SIO: u16 = 7;
pub const IRQ_DMA0: u16 = 8;
pub const IRQ_DMA1: u16 = 9;
pub const IRQ_DMA2: u16 = 10;
pub const IRQ_DMA3: u16 = 11;
pub const IRQ_KEYPAD: u16 = 12;
pub const IRQ_GAMEPAK: u16 = 13;
