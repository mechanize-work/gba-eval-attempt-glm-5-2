#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

                }
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
