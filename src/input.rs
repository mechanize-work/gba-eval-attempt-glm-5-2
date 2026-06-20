// Input/Keypad handling
pub struct Input {
    pub keys: u32, // active-high
}

impl Input {
    pub fn new() -> Self {
        Input { keys: 0 }
    }

    pub fn set_keys(&mut self, keys: u32) {
        self.keys = keys;
    }

    // Returns KEYINPUT value (active-low: 0 = pressed)
    pub fn keyinput(&self) -> u16 {
        let mut val: u16 = 0x3FF; // all released
        if self.keys & (1 << 0) != 0 { val &= !(1 << 0); } // A
        if self.keys & (1 << 1) != 0 { val &= !(1 << 1); } // B
        if self.keys & (1 << 2) != 0 { val &= !(1 << 2); } // Select
        if self.keys & (1 << 3) != 0 { val &= !(1 << 3); } // Start
        if self.keys & (1 << 4) != 0 { val &= !(1 << 4); } // Right
        if self.keys & (1 << 5) != 0 { val &= !(1 << 5); } // Left
        if self.keys & (1 << 6) != 0 { val &= !(1 << 6); } // Up
        if self.keys & (1 << 7) != 0 { val &= !(1 << 7); } // Down
        if self.keys & (1 << 8) != 0 { val &= !(1 << 8); } // R
        if self.keys & (1 << 9) != 0 { val &= !(1 << 9); } // L
        val
    }
}
