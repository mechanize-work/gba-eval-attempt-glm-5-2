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

// GBA Memory Map:
// 0x00000000 - 0x00003FFF  BIOS (16KB)
// 0x02000000 - 0x0203FFFF  EWRAM (256KB)
// 0x03000000 - 0x03007FFF  IWRAM (32KB)
// 0x04000000 - 0x040003FE  IO Registers
// 0x05000000 - 0x050003FF  Palette RAM (1KB)
// 0x06000000 - 0x06017FFF  VRAM (96KB)
// 0x07000000 - 0x070003FF  OAM (1KB)
// 0x08000000 - 0x09FFFFFF  Game Pak ROM (up to 32MB)
// 0x0A000000 -             Game Pak ROM (mirror)

pub const BIOS_SIZE: usize = 16 * 1024;
pub const EWRAM_SIZE: usize = 256 * 1024;
pub const IWRAM_SIZE: usize = 32 * 1024;
pub const IO_SIZE: usize = 0x400;
pub const PALETTE_SIZE: usize = 1024;
pub const VRAM_SIZE: usize = 96 * 1024;
pub const OAM_SIZE: usize = 1024;
pub const ROM_MAX_SIZE: usize = 32 * 1024 * 1024;

pub const BIOS_BASE: u32 = 0x0000_0000;
pub const EWRAM_BASE: u32 = 0x0200_0000;
pub const IWRAM_BASE: u32 = 0x0300_0000;
pub const IO_BASE: u32 = 0x0400_0000;
pub const PALETTE_BASE: u32 = 0x0500_0000;
pub const VRAM_BASE: u32 = 0x0600_0000;
pub const OAM_BASE: u32 = 0x0700_0000;
pub const ROM_BASE: u32 = 0x0800_0000;

// IO Register offsets
pub const REG_DISPCNT: u32 = 0x00;
pub const REG_GREENSWP: u32 = 0x02;
pub const REG_DISPSTAT: u32 = 0x04;
pub const REG_VCOUNT: u32 = 0x06;
pub const REG_BG0CNT: u32 = 0x08;
pub const REG_BG1CNT: u32 = 0x0A;
pub const REG_BG2CNT: u32 = 0x0C;
pub const REG_BG3CNT: u32 = 0x0E;
pub const REG_BG0HOFS: u32 = 0x10;
pub const REG_BG0VOFS: u32 = 0x12;
pub const REG_BG1HOFS: u32 = 0x14;
pub const REG_BG1VOFS: u32 = 0x16;
pub const REG_BG2HOFS: u32 = 0x18;
pub const REG_BG2VOFS: u32 = 0x1A;
pub const REG_BG3HOFS: u32 = 0x1C;
pub const REG_BG3VOFS: u32 = 0x1E;
pub const REG_BG2PA: u32 = 0x20;
pub const REG_BG2PB: u32 = 0x22;
pub const REG_BG2PC: u32 = 0x24;
pub const REG_BG2PD: u32 = 0x26;
pub const REG_BG2X: u32 = 0x28;
pub const REG_BG2X_L: u32 = 0x28;
pub const REG_BG2X_H: u32 = 0x2A;
pub const REG_BG2Y: u32 = 0x2C;
pub const REG_BG2Y_L: u32 = 0x2C;
pub const REG_BG2Y_H: u32 = 0x2E;
pub const REG_BG3PA: u32 = 0x30;
pub const REG_BG3PB: u32 = 0x32;
pub const REG_BG3PC: u32 = 0x34;
pub const REG_BG3PD: u32 = 0x36;
pub const REG_BG3X: u32 = 0x38;
pub const REG_BG3X_L: u32 = 0x38;
pub const REG_BG3X_H: u32 = 0x3A;
pub const REG_BG3Y: u32 = 0x3C;
pub const REG_BG3Y_L: u32 = 0x3C;
pub const REG_BG3Y_H: u32 = 0x3E;
pub const REG_WIN0H: u32 = 0x40;
pub const REG_WIN1H: u32 = 0x42;
pub const REG_WIN0V: u32 = 0x44;
pub const REG_WIN1V: u32 = 0x46;
pub const REG_WININ: u32 = 0x48;
pub const REG_WINOUT: u32 = 0x4A;
pub const REG_MOSAIC: u32 = 0x4C;
pub const REG_BLDCNT: u32 = 0x50;
pub const REG_BLDALPHA: u32 = 0x52;
pub const REG_BLDY: u32 = 0x54;

// Sound registers
pub const REG_SOUND1CNT_L: u32 = 0x60;
pub const REG_SOUND1CNT_H: u32 = 0x62;
pub const REG_SOUND1CNT_X: u32 = 0x64;
pub const REG_SOUND2CNT_L: u32 = 0x68;
pub const REG_SOUND2CNT_H: u32 = 0x6C;
pub const REG_SOUND3CNT_L: u32 = 0x70;
pub const REG_SOUND3CNT_H: u32 = 0x72;
pub const REG_SOUND3CNT_X: u32 = 0x74;
pub const REG_SOUND4CNT_L: u32 = 0x78;
pub const REG_SOUND4CNT_H: u32 = 0x7C;
pub const REG_SOUNDCNT_L: u32 = 0x80;
pub const REG_SOUNDCNT_H: u32 = 0x82;
pub const REG_SOUNDCNT_X: u32 = 0x84;
pub const REG_SOUNDBIAS: u32 = 0x88;
pub const REG_WAVE_RAM: u32 = 0x90;
pub const REG_FIFO_A: u32 = 0xA0;
pub const REG_FIFO_B: u32 = 0xA4;

// DMA registers
pub const REG_DMA0SAD: u32 = 0xB0;
pub const REG_DMA0DAD: u32 = 0xB4;
pub const REG_DMA0CNT: u32 = 0xB8;
pub const REG_DMA1SAD: u32 = 0xBC;
pub const REG_DMA1DAD: u32 = 0xC0;
pub const REG_DMA1CNT: u32 = 0xC4;
pub const REG_DMA2SAD: u32 = 0xC8;
pub const REG_DMA2DAD: u32 = 0xCC;
pub const REG_DMA2CNT: u32 = 0xD0;
pub const REG_DMA3SAD: u32 = 0xD4;
pub const REG_DMA3DAD: u32 = 0xD8;
pub const REG_DMA3CNT: u32 = 0xDC;

// Timer registers
pub const REG_TM0CNT: u32 = 0x100;
pub const REG_TM1CNT: u32 = 0x104;
pub const REG_TM2CNT: u32 = 0x108;
pub const REG_TM3CNT: u32 = 0x10C;

// Serial/Keypad
pub const REG_SIODATA8: u32 = 0x12A;
pub const REG_RCNT: u32 = 0x134;
pub const REG_JOYCNT: u32 = 0x140;
pub const REG_SIODATA32: u32 = 0x150;
pub const REG_JOYBUS: u32 = 0x158;
pub const REG_JOYSTAT: u32 = 0x158;
pub const REG_KEYINPUT: u32 = 0x130;
pub const REG_KEYCNT: u32 = 0x132;

// Interrupt registers
pub const REG_IE: u32 = 0x200;
pub const REG_IF: u32 = 0x202;
pub const REG_WAITCNT: u32 = 0x204;
pub const REG_IME: u32 = 0x208;

// PostBoot
pub const REG_POSTFLG: u32 = 0x300;
pub const REG_HALTCNT: u32 = 0x301;

pub struct Memory {
    pub bios: Box<[u8; BIOS_SIZE]>,
    pub ewram: Box<[u8; EWRAM_SIZE]>,
    pub iwram: Box<[u8; IWRAM_SIZE]>,
    pub io: Box<[u8; IO_SIZE]>,
    pub palette: Box<[u8; PALETTE_SIZE]>,
    pub vram: Box<[u8; VRAM_SIZE]>,
    pub oam: Box<[u8; OAM_SIZE]>,
    pub rom: Box<[u8]>,
    pub rom_size: usize,
    pub openbus: u32,
    pub dma_cnt: [u32; 4],    // DMA control registers (cached for emulator access)
    pub timer_cnt: [u16; 4],  // Timer control registers
    pub timer_data: [u16; 4], // Timer reload values
    pub apu_regs: [u16; 0x30], // Sound registers 0x60-0x8F
    pub soundbias: u16,
    pub haltcnt: u8,
    pub waitcnt: u16,
}

impl Memory {
    pub fn new() -> Self {
        let rom = {
            let mut v = Vec::new();
            v.resize(ROM_MAX_SIZE, 0u8);
            v.into_boxed_slice()
        };
        Memory {
            bios: Box::new([0u8; BIOS_SIZE]),
            ewram: Box::new([0u8; EWRAM_SIZE]),
            iwram: Box::new([0u8; IWRAM_SIZE]),
            io: Box::new([0u8; IO_SIZE]),
            palette: Box::new([0u8; PALETTE_SIZE]),
            vram: Box::new([0u8; VRAM_SIZE]),
            oam: Box::new([0u8; OAM_SIZE]),
            rom,
            rom_size: 0,
            openbus: 0,
            dma_cnt: [0; 4],
            timer_cnt: [0; 4],
            timer_data: [0; 4],
            apu_regs: [0; 0x30],
            soundbias: 0x200,
            haltcnt: 0,
            waitcnt: 0,
        }
    }

    pub fn load_bios(&mut self, bios: &[u8]) {
        let len = bios.len().min(BIOS_SIZE);
        self.bios[..len].copy_from_slice(&bios[..len]);
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let len = rom.len().min(ROM_MAX_SIZE);
        self.rom[..len].copy_from_slice(rom);
        self.rom_size = len;
    }

    #[inline]
    fn rom_read(&self, addr: u32, size: usize) -> u32 {
        let mask = if self.rom_size > 1 {
            // ROM wraps based on next power of 2 up to 32MB
            let mut m = 1usize;
            while m < self.rom_size {
                m <<= 1;
            }
            m - 1
        } else {
            0xFFFFFFFF
        };
        let a = (addr as usize) & mask;
        let mut val: u32 = 0;
        for i in 0..size {
            let off = a.wrapping_add(i);
            if off < self.rom_size {
                val |= (self.rom[off] as u32) << (i * 8);
            } else if self.rom_size > 0 {
                // Open bus from ROM
                val |= if (a + i) & 1 == 1 { 0xFF } else { 0 } << (i * 8);
            }
        }
        val
    }

    #[inline]
    pub fn read_byte(&mut self, addr: u32) -> u8 {
        match addr >> 24 {
            0x00 => { // BIOS
                let a = (addr as usize) & (BIOS_SIZE - 1);
                self.bios[a]
            }
            0x02 => { // EWRAM
                let a = (addr as usize) & (EWRAM_SIZE - 1);
                self.ewram[a]
            }
            0x03 | 0x04 => { // IWRAM (0x03000000-0x03FFFFFF) and its mirror at 0x04000000 region
                // Actually 0x04 is IO, not IWRAM. Fix this.
                if addr >= 0x0400_0000 && addr < 0x0400_0400 {
                    self.io_read(addr)
                } else {
                    let a = (addr as usize) & (IWRAM_SIZE - 1);
                    self.iwram[a]
                }
            }
            0x05 => { // Palette
                let a = (addr as usize) & (PALETTE_SIZE - 1);
                self.palette[a]
            }
            0x06 => { // VRAM
                let a = (addr as usize) & (VRAM_SIZE - 1);
                self.vram[a]
            }
            0x07 => { // OAM
                let a = (addr as usize) & (OAM_SIZE - 1);
                self.oam[a]
            }
            0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D => { // ROM
                self.rom_read(addr & 0x01FF_FFFF, 1) as u8
            }
            _ => {
                // 0x03000000-0x03FFFFFF: IWRAM
                if addr >= 0x0300_0000 && addr < 0x0400_0000 {
                    let a = (addr as usize) & (IWRAM_SIZE - 1);
                    self.iwram[a]
                } else {
                    0
                }
            }
        }
    }

    #[inline]
    pub fn read_half(&mut self, addr: u32) -> u16 {
        // For simplicity, read two bytes. Optimized paths can be added later.
        // But IO registers need special handling.
        if addr >= 0x0400_0000 && addr < 0x0400_0400 {
            return self.io_read_half(addr);
        }
        let b0 = self.read_byte(addr) as u16;
        let b1 = self.read_byte(addr.wrapping_add(1)) as u16;
        b0 | (b1 << 8)
    }

    #[inline]
    pub fn read_word(&mut self, addr: u32) -> u32 {
        // IO registers need special handling
        if addr >= 0x0400_0000 && addr < 0x0400_0400 {
            let h0 = self.io_read_half(addr) as u32;
            let h1 = self.io_read_half(addr.wrapping_add(2)) as u32;
            return h0 | (h1 << 16);
        }
        // ARM unaligned access: read aligned word and rotate
        let aligned = addr & !3;
        let b0 = self.read_byte(aligned) as u32;
        let b1 = self.read_byte(aligned.wrapping_add(1)) as u32;
        let b2 = self.read_byte(aligned.wrapping_add(2)) as u32;
        let b3 = self.read_byte(aligned.wrapping_add(3)) as u32;
        let val = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        let rotate = (addr & 3) * 8;
        val.rotate_right(rotate)
    }

    #[inline]
    pub fn write_byte(&mut self, addr: u32, val: u8) {
        match addr >> 24 {
            0x00 => { // BIOS - read-only after boot (BIOSPROT)
                // Writes to BIOS are ignored after boot
            }
            0x02 => { // EWRAM
                let a = (addr as usize) & (EWRAM_SIZE - 1);
                self.ewram[a] = val;
            }
            0x03 => { // IWRAM
                let a = (addr as usize) & (IWRAM_SIZE - 1);
                self.iwram[a] = val;
            }
            0x04 => { // IO
                if addr < 0x0400_0400 {
                    self.io_write_byte(addr, val);
                }
            }
            0x05 => { // Palette
                let a = (addr as usize) & (PALETTE_SIZE - 1);
                self.palette[a] = val;
            }
            0x06 => { // VRAM
                let a = (addr as usize) & (VRAM_SIZE - 1);
                self.vram[a] = val;
            }
            0x07 => { // OAM
                let a = (addr as usize) & (OAM_SIZE - 1);
                self.oam[a] = val;
            }
            _ => {}
        }
    }

    #[inline]
    pub fn write_half(&mut self, addr: u32, val: u16) {
        if addr >= 0x0400_0000 && addr < 0x0400_0400 {
            self.io_write_half(addr, val);
            return;
        }
        self.write_byte(addr, (val & 0xFF) as u8);
        self.write_byte(addr.wrapping_add(1), ((val >> 8) & 0xFF) as u8);
    }

    #[inline]
    pub fn write_word(&mut self, addr: u32, val: u32) {
        if addr >= 0x0400_0000 && addr < 0x0400_0400 {
            self.io_write_half(addr, (val & 0xFFFF) as u16);
            self.io_write_half(addr.wrapping_add(2), ((val >> 16) & 0xFFFF) as u16);
            return;
        }
        self.write_byte(addr, (val & 0xFF) as u8);
        self.write_byte(addr.wrapping_add(1), ((val >> 8) & 0xFF) as u8);
        self.write_byte(addr.wrapping_add(2), ((val >> 16) & 0xFF) as u8);
        self.write_byte(addr.wrapping_add(3), ((val >> 24) & 0xFF) as u8);
    }

    // IO register access - will be hooked up by the emulator
    // For now, simple storage
    #[inline]
    fn io_read(&mut self, addr: u32) -> u8 {
        let a = ((addr - IO_BASE) as usize) & (IO_SIZE - 1);
        self.io[a]
    }

    #[inline]
    fn io_read_half(&mut self, addr: u32) -> u16 {
        // Special read-only/write-only/composite registers
        let off = (addr - IO_BASE) as u16;
        match off {
            0x130 => {
                // Active-low: 0 means pressed
                let keys = self.io_read_half_internal(0x130);
                // keyinput is read-only, returns the complement of pressed keys
                keys
            }
            _ => {
                let a = (off as usize) & (IO_SIZE - 1);
                (self.io[a] as u16) | ((self.io[(a + 1) & (IO_SIZE - 1)] as u16) << 8)
            }
        }
    }

    #[inline]
    fn io_read_half_internal(&self, off: u16) -> u16 {
        let a = (off as usize) & (IO_SIZE - 1);
        (self.io[a] as u16) | ((self.io[(a + 1) & (IO_SIZE - 1)] as u16) << 8)
    }

    #[inline]
    fn io_write_half(&mut self, addr: u32, val: u16) {
        let a = ((addr - IO_BASE) as usize) & (IO_SIZE - 1);
        if a == 0x202 {
            let current = (self.io[a] as u16) | ((self.io[(a + 1) & (IO_SIZE - 1)] as u16) << 8);
            let new_val = current & !val;
            self.io[a] = (new_val & 0xFF) as u8;
            self.io[(a + 1) & (IO_SIZE - 1)] = ((new_val >> 8) & 0xFF) as u8;
            return;
        }
        if a == 0x130 || a == 0x131 { return; }
        if a == 0x204 {
            // WAITCNT
            self.io[a] = (val & 0xFF) as u8;
            self.io[(a + 1) & (IO_SIZE - 1)] = ((val >> 8) & 0xFF) as u8;
            self.waitcnt = val;
            return;
        }
        if a == 0x300 {
            self.io[a] = (val & 0xFF) as u8;
            self.io[(a + 1) & (IO_SIZE - 1)] = ((val >> 8) & 0xFF) as u8;
            self.haltcnt = (val & 0xFF) as u8;
            return;
        }
        // DMA registers: cache CNT for emulator
        if a >= 0xB0 && a <= 0xDF {
            self.io[a] = (val & 0xFF) as u8;
            self.io[(a + 1) & (IO_SIZE - 1)] = ((val >> 8) & 0xFF) as u8;
            for ch in 0..4 {
                let cnt_off = 0xB8 + ch * 0x0C;
                if a == cnt_off {
                    let hi = (self.io[cnt_off + 2] as u16) | ((self.io[cnt_off + 3] as u16) << 8);
                    self.dma_cnt[ch] = (val as u32) | ((hi as u32) << 16);
                } else if a == cnt_off + 2 {
                    let lo = (self.io[cnt_off] as u16) | ((self.io[cnt_off + 1] as u16) << 8);
                    self.dma_cnt[ch] = (lo as u32) | ((val as u32) << 16);
                }
            }

            return;
        }
        // Timer registers
        if a >= 0x100 && a <= 0x10F {
            self.io[a] = (val & 0xFF) as u8;
            self.io[(a + 1) & (IO_SIZE - 1)] = ((val >> 8) & 0xFF) as u8;
            let tm = (a - 0x100) / 4;
            if a == 0x100 + tm * 4 { self.timer_data[tm] = val; }
            if a == 0x102 + tm * 4 { self.timer_cnt[tm] = val; }
            return;
        }
        // Sound registers
        if a >= 0x60 && a <= 0x8F {
            self.io[a] = (val & 0xFF) as u8;
            self.io[(a + 1) & (IO_SIZE - 1)] = ((val >> 8) & 0xFF) as u8;
            let idx = (a - 0x60) / 2;
            if idx < 0x18 { self.apu_regs[idx] = val; }
            if a == 0x88 { self.soundbias = val; }
            return;
        }
        self.io[a] = (val & 0xFF) as u8;
        self.io[(a + 1) & (IO_SIZE - 1)] = ((val >> 8) & 0xFF) as u8;
    }

    #[inline]
    fn io_write_byte(&mut self, addr: u32, val: u8) {
        let a = ((addr - IO_BASE) as usize) & (IO_SIZE - 1);
        // KEYINPUT is read-only
        if a == 0x130 || a == 0x131 {
            return;
        }
        // IF is write-to-clear (byte access)
        if a == 0x202 || a == 0x203 {
            self.io[a] &= !val;
            return;
        }
        self.io[a] = val;
    }

    pub fn reset(&mut self) {
        for b in self.ewram.iter_mut() { *b = 0; }
        for b in self.iwram.iter_mut() { *b = 0; }
        for b in self.io.iter_mut() { *b = 0; }
        for b in self.palette.iter_mut() { *b = 0; }
        for b in self.vram.iter_mut() { *b = 0; }
        for b in self.oam.iter_mut() { *b = 0; }
    }

    // Direct access helpers for the PPU and other components
    pub fn read_vram_byte(&self, addr: u32) -> u8 {
        let a = (addr as usize) & (VRAM_SIZE - 1);
        self.vram[a]
    }

    pub fn read_vram_half(&self, addr: u32) -> u16 {
        let a = (addr as usize) & (VRAM_SIZE - 1);
        (self.vram[a] as u16) | ((self.vram[a + 1] as u16) << 8)
    }

    pub fn read_palette_half(&self, addr: u32) -> u16 {
        let a = (addr as usize) & (PALETTE_SIZE - 1);
        (self.palette[a] as u16) | ((self.palette[a + 1] as u16) << 8)
    }

    pub fn read_oam_half(&self, addr: u32) -> u16 {
        let a = (addr as usize) & (OAM_SIZE - 1);
        (self.oam[a] as u16) | ((self.oam[a + 1] as u16) << 8)
    }

    pub fn read_iwram_half(&self, addr: u32) -> u16 {
        let a = (addr as usize) & (IWRAM_SIZE - 1);
        (self.iwram[a] as u16) | ((self.iwram[a + 1] as u16) << 8)
    }

    pub fn read_iwram_word(&self, addr: u32) -> u32 {
        let a = (addr as usize) & (IWRAM_SIZE - 1);
        (self.iwram[a] as u32)
            | ((self.iwram[a + 1] as u32) << 8)
            | ((self.iwram[a + 2] as u32) << 16)
            | ((self.iwram[a + 3] as u32) << 24)
    }

    pub fn write_iwram_half(&mut self, addr: u32, val: u16) {
        let a = (addr as usize) & (IWRAM_SIZE - 1);
        self.iwram[a] = (val & 0xFF) as u8;
        self.iwram[a + 1] = ((val >> 8) & 0xFF) as u8;
    }

    pub fn write_iwram_word(&mut self, addr: u32, val: u32) {
        let a = (addr as usize) & (IWRAM_SIZE - 1);
        self.iwram[a] = (val & 0xFF) as u8;
        self.iwram[a + 1] = ((val >> 8) & 0xFF) as u8;
        self.iwram[a + 2] = ((val >> 16) & 0xFF) as u8;
        self.iwram[a + 3] = ((val >> 24) & 0xFF) as u8;
    }

    pub fn read_ewram_half(&self, addr: u32) -> u16 {
        let a = (addr as usize) & (EWRAM_SIZE - 1);
        (self.ewram[a] as u16) | ((self.ewram[a + 1] as u16) << 8)
    }

    pub fn read_ewram_word(&self, addr: u32) -> u32 {
        let a = (addr as usize) & (EWRAM_SIZE - 1);
        (self.ewram[a] as u32)
            | ((self.ewram[a + 1] as u32) << 8)
            | ((self.ewram[a + 2] as u32) << 16)
            | ((self.ewram[a + 3] as u32) << 24)
    }

    pub fn write_ewram_half(&mut self, addr: u32, val: u16) {
        let a = (addr as usize) & (EWRAM_SIZE - 1);
        self.ewram[a] = (val & 0xFF) as u8;
        self.ewram[a + 1] = ((val >> 8) & 0xFF) as u8;
    }

    pub fn write_ewram_word(&mut self, addr: u32, val: u32) {
        let a = (addr as usize) & (EWRAM_SIZE - 1);
        self.ewram[a] = (val & 0xFF) as u8;
        self.ewram[a + 1] = ((val >> 8) & 0xFF) as u8;
        self.ewram[a + 2] = ((val >> 16) & 0xFF) as u8;
        self.ewram[a + 3] = ((val >> 24) & 0xFF) as u8;
    }

    pub fn read_rom_word(&self, addr: u32) -> u32 {
        self.rom_read(addr & 0x01FF_FFFF, 4)
    }

    pub fn read_rom_half(&self, addr: u32) -> u16 {
        self.rom_read(addr & 0x01FF_FFFF, 2) as u16
    }

    pub fn read_rom_byte(&self, addr: u32) -> u8 {
        self.rom_read(addr & 0x01FF_FFFF, 1) as u8
    }
}
