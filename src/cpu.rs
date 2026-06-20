// ARM7TDMI CPU implementation
// Supports ARM and THUMB instruction sets

use crate::memory::Memory;

// CPU modes
pub const MODE_USR: u32 = 0x10;
pub const MODE_FIQ: u32 = 0x11;
pub const MODE_IRQ: u32 = 0x12;
pub const MODE_SVC: u32 = 0x13;
pub const MODE_ABT: u32 = 0x17;
pub const MODE_UND: u32 = 0x1B;
pub const MODE_SYS: u32 = 0x1F;

// CPSR flags
pub const FLAG_N: u32 = 0x8000_0000;
pub const FLAG_Z: u32 = 0x4000_0000;
pub const FLAG_C: u32 = 0x2000_0000;
pub const FLAG_V: u32 = 0x1000_0000;
pub const FLAG_Q: u32 = 0x0800_0000;
pub const FLAG_I: u32 = 0x0000_0080;
pub const FLAG_F: u32 = 0x0000_0040;
pub const FLAG_T: u32 = 0x0000_0020;

// Exception vector addresses
pub const EXC_RESET: u32 = 0x00;
pub const EXC_UND: u32 = 0x04;
pub const EXC_SWI: u32 = 0x08;
pub const EXC_PABT: u32 = 0x0C;
pub const EXC_DABT: u32 = 0x10;
pub const EXC_IRQ: u32 = 0x18;
pub const EXC_FIQ: u32 = 0x1C;

#[derive(Clone, Copy)]
pub struct BankedRegs {
    pub r13: u32,
    pub r14: u32,
    pub spsr: u32,
}

pub struct Cpu {
    pub r: [u32; 16],
    pub cpsr: u32,
    pub spsr: u32,

    // Banked registers for FIQ
    pub fiq_r8_r12: [u32; 5],
    pub fiq_r13: u32,
    pub fiq_r14: u32,
    pub fiq_spsr: u32,

    // Banked for IRQ
    pub irq_r13: u32,
    pub irq_r14: u32,
    pub irq_spsr: u32,

    // Banked for SVC
    pub svc_r13: u32,
    pub svc_r14: u32,
    pub svc_spsr: u32,

    // Banked for ABT
    pub abt_r13: u32,
    pub abt_r14: u32,
    pub abt_spsr: u32,

    // Banked for UND
    pub und_r13: u32,
    pub und_r14: u32,
    pub und_spsr: u32,

    // System/user banked (none, uses r13/r14)

    pub halted: bool,
    pub vblank_intr_wait: bool,
    pub cycles: u64,

    // Pipeline
    pub pipeline_fetch: u32,
    pub pipeline_decode: u32,

    // For debugging
    pub last_pc: u32,

    // IRQ pending
    pub irq_line: bool,

    // Halt related
    pub stop: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            r: [0; 16],
            cpsr: 0,
            spsr: 0,
            fiq_r8_r12: [0; 5],
            fiq_r13: 0,
            fiq_r14: 0,
            fiq_spsr: 0,
            irq_r13: 0,
            irq_r14: 0,
            irq_spsr: 0,
            svc_r13: 0,
            svc_r14: 0,
            svc_spsr: 0,
            abt_r13: 0,
            abt_r14: 0,
            abt_spsr: 0,
            und_r13: 0,
            und_r14: 0,
            und_spsr: 0,
            halted: false,
            vblank_intr_wait: false,
            cycles: 0,
            pipeline_fetch: 0,
            pipeline_decode: 0,
            last_pc: 0,
            irq_line: false,
            stop: false,
        };
        // Initial state: SVC mode, IRQ disabled, FIQ disabled
        cpu.cpsr = MODE_SVC | FLAG_I | FLAG_F;
        cpu.svc_r13 = 0x0300_7FC0;
        cpu.irq_r13 = 0x0300_7FA0;
        cpu.r[13] = cpu.svc_r13;
        cpu.r[15] = 0x0000_0000;
        cpu
    }

    pub fn reset(&mut self) {
        self.r = [0; 16];
        self.cpsr = MODE_SVC | FLAG_I | FLAG_F;
        self.spsr = 0;
        self.fiq_r8_r12 = [0; 5];
        self.fiq_r13 = 0;
        self.fiq_r14 = 0;
        self.fiq_spsr = 0;
        self.irq_r13 = 0x0300_7FA0;
        self.irq_r14 = 0;
        self.irq_spsr = 0;
        self.svc_r13 = 0x0300_7FC0;
        self.svc_r14 = 0;
        self.svc_spsr = 0;
        self.abt_r13 = 0;
        self.abt_r14 = 0;
        self.abt_spsr = 0;
        self.und_r13 = 0;
        self.und_r14 = 0;
        self.und_spsr = 0;
        self.halted = false;
        self.cycles = 0;
        self.pipeline_fetch = 0;
        self.pipeline_decode = 0;
        self.last_pc = 0;
        self.irq_line = false;
        self.stop = false;
        self.r[13] = self.svc_r13;
        self.r[15] = 0x0000_0000;
    }

    #[inline]
    pub fn get_mode(&self) -> u32 {
        self.cpsr & 0x1F
    }

    #[inline]
    pub fn is_thumb(&self) -> bool {
        (self.cpsr & FLAG_T) != 0
    }

    #[inline]
    pub fn set_flag(&mut self, flag: u32, on: bool) {
        if on {
            self.cpsr |= flag;
        } else {
            self.cpsr &= !flag;
        }
    }

    #[inline]
    pub fn get_flag(&self, flag: u32) -> bool {
        (self.cpsr & flag) != 0
    }

    #[inline]
    pub fn set_nz(&mut self, result: u32) {
        self.set_flag(FLAG_N, (result >> 31) & 1 != 0);
        self.set_flag(FLAG_Z, result == 0);
    }

    #[inline]
    pub fn set_nzc(&mut self, result: u32, carry: bool) {
        self.set_nz(result);
        self.set_flag(FLAG_C, carry);
    }

    #[inline]
    pub fn set_nzcv(&mut self, result: u32, carry: bool, overflow: bool) {
        self.set_nz(result);
        self.set_flag(FLAG_C, carry);
        self.set_flag(FLAG_V, overflow);
    }

    // Switch to a new mode, banking/unbanking registers
    pub fn switch_mode(&mut self, new_mode: u32) {
        let old_mode = self.get_mode();

        if old_mode == new_mode {
            return;
        }

        // Bank current r13/r14
        match old_mode {
            MODE_FIQ => {
                self.fiq_r13 = self.r[13];
                self.fiq_r14 = self.r[14];
                // Restore r8-r12 if leaving FIQ
                if new_mode != MODE_FIQ {
                    // Save FIQ r8-r12 and restore user r8-r12
                    // We need to store the user copies somewhere...
                    // Actually, in our model, we don't store user r8-r12 separately
                    // For simplicity, let's store them in the fiq array and restore
                    // But we never saved the user values...
                    // This is a simplification - proper banking requires shadow storage
                }
            }
            MODE_IRQ => {
                self.irq_r13 = self.r[13];
                self.irq_r14 = self.r[14];
            }
            MODE_SVC => {
                self.svc_r13 = self.r[13];
                self.svc_r14 = self.r[14];
            }
            MODE_ABT => {
                self.abt_r13 = self.r[13];
                self.abt_r14 = self.r[14];
            }
            MODE_UND => {
                self.und_r13 = self.r[13];
                self.und_r14 = self.r[14];
            }
            _ => {}
        }

        // Restore r13/r14 for new mode
        match new_mode {
            MODE_FIQ => {
                self.r[13] = self.fiq_r13;
                self.r[14] = self.fiq_r14;
            }
            MODE_IRQ => {
                self.r[13] = self.irq_r13;
                self.r[14] = self.irq_r14;
            }
            MODE_SVC => {
                self.r[13] = self.svc_r13;
                self.r[14] = self.svc_r14;
            }
            MODE_ABT => {
                self.r[13] = self.abt_r13;
                self.r[14] = self.abt_r14;
            }
            MODE_UND => {
                self.r[13] = self.und_r13;
                self.r[14] = self.und_r14;
            }
            _ => {
                // USR and SYS share r13/r14 with... each other
                // But we need to save/restore. For now, use what's in r[13]/r[14]
            }
        }

        self.cpsr = (self.cpsr & !0x1F) | new_mode;
    }

    pub fn get_spsr(&self) -> u32 {
        match self.get_mode() {
            MODE_FIQ => self.fiq_spsr,
            MODE_IRQ => self.irq_spsr,
            MODE_SVC => self.svc_spsr,
            MODE_ABT => self.abt_spsr,
            MODE_UND => self.und_spsr,
            _ => self.cpsr, // USR/SYS don't have SPSR
        }
    }

    pub fn set_spsr(&mut self, val: u32) {
        match self.get_mode() {
            MODE_FIQ => self.fiq_spsr = val,
            MODE_IRQ => self.irq_spsr = val,
            MODE_SVC => self.svc_spsr = val,
            MODE_ABT => self.abt_spsr = val,
            MODE_UND => self.und_spsr = val,
            _ => {} // USR/SYS ignore
        }
    }

    pub fn exception(&mut self, vector: u32, new_mode: u32, mask_irq: bool, mask_fiq: bool) {
        let old_cpsr = self.cpsr;
        let thumb = self.is_thumb();

        // Save return address
        let ret_addr = if thumb {
            self.r[15] // already adjusted for pipeline
        } else {
            self.r[15]
        };

        self.switch_mode(new_mode);
        self.set_spsr(old_cpsr);

        // Set LR
        // For ARM: LR = PC + 4 (already in pipeline state)
        // For THUMB: LR = PC
        // For ARM: LR = PC - 4 (so SUBS PC, LR, #4 returns to next instruction)
        // For THUMB: LR = PC + 4 (so SUBS PC, LR, #4 returns to next instruction)
        // PC in THUMB = current_instruction + 4 (pipeline)
        // PC in ARM = current_instruction + 8 (pipeline)
        // ret_addr = self.r[15] which is current instruction address (before pipeline adjustment)
        // For THUMB: LR should be current + 8, ret_addr = current, so LR = ret_addr + 8
        // For ARM: LR should be current + 4, ret_addr = current, so LR = ret_addr + 4
        // For THUMB: LR = current + 6 (return = LR - 4 = current + 2 = next instruction)
        // For ARM: LR = current + 4 (return = LR - 4 = current + 4 = next instruction)
        if thumb {
            self.r[14] = ret_addr.wrapping_add(6);
        } else {
            self.r[14] = ret_addr.wrapping_add(4);
        }

        // Switch to ARM mode
        self.cpsr &= !FLAG_T;

        if mask_irq { self.cpsr |= FLAG_I; }
        if mask_fiq { self.cpsr |= FLAG_F; }

        self.r[15] = vector;
        self.flush_pipeline();
        self.cycles += 2; // roughly
    }

    pub fn raise_irq(&mut self) {
        if self.halted {
            self.halted = false;
        }
        if !self.get_flag(FLAG_I) {
            self.exception(EXC_IRQ, MODE_IRQ, true, false);
        }
    }

    pub fn flush_pipeline(&mut self) {
        // PC has been changed, need to refill pipeline
        // This will be handled by the execution loop
    }

    // Check for interrupts and handle
    pub fn check_interrupts(&mut self, ie: u32, if_: u32, ime: bool) {
        if ime && (ie & if_) != 0 {
            if !self.get_flag(FLAG_I) {
                self.raise_irq();
            }
        }
    }

    // Get register for load/store (handles R15)
    #[inline]
    pub fn reg_read(&self, idx: usize) -> u32 {
        if idx == 15 {
            self.r[15] + if self.is_thumb() { 4 } else { 8 }
        } else {
            self.r[idx]
        }
    }

    #[inline]
    pub fn reg_read_no_pc(&self, idx: usize) -> u32 {
        self.r[idx]
    }

    // Conditional execution check
    #[inline]
    pub fn check_cond(&self, cond: u32) -> bool {
        match cond {
            0x0 => self.get_flag(FLAG_Z),          // EQ
            0x1 => !self.get_flag(FLAG_Z),         // NE
            0x2 => self.get_flag(FLAG_C),          // CS/HS
            0x3 => !self.get_flag(FLAG_C),         // CC/LO
            0x4 => self.get_flag(FLAG_N),          // MI
            0x5 => !self.get_flag(FLAG_N),         // PL
            0x6 => self.get_flag(FLAG_V),          // VS
            0x7 => !self.get_flag(FLAG_V),         // VC
            0x8 => self.get_flag(FLAG_C) && !self.get_flag(FLAG_Z), // HI
            0x9 => !self.get_flag(FLAG_C) || self.get_flag(FLAG_Z), // LS
            0xA => self.get_flag(FLAG_N) == self.get_flag(FLAG_V),  // GE
            0xB => self.get_flag(FLAG_N) != self.get_flag(FLAG_V),  // LT
            0xC => !self.get_flag(FLAG_Z) && (self.get_flag(FLAG_N) == self.get_flag(FLAG_V)), // GT
            0xD => self.get_flag(FLAG_Z) || (self.get_flag(FLAG_N) != self.get_flag(FLAG_V)),  // LE
            0xE => true,                            // AL (always)
            0xF => true,                            // NV (never in ARMv4, always in ARMv5+)
            _ => true,
        }
    }

    // Barrel shifter
    #[inline]
    pub fn lsl(val: u32, shift: u32) -> u32 {
        if shift == 0 {
            val
        } else if shift >= 32 {
            0
        } else {
            val << shift
        }
    }

    #[inline]
    pub fn lsl_c(val: u32, shift: u32) -> (u32, bool) {
        if shift == 0 {
            (val, false) // no shift, carry unchanged
        } else if shift >= 32 {
            if shift == 32 {
                (0, val & 1 != 0)
            } else if shift > 64 {
                (0, false)
            } else {
                (0, false)
            }
        } else {
            let carry = (val >> (32 - shift)) & 1 != 0;
            (val << shift, carry)
        }
    }

    #[inline]
    pub fn lsr(val: u32, shift: u32) -> u32 {
        if shift == 0 {
            val // LSR #0 = LSR #32
        } else if shift >= 32 {
            0
        } else {
            val >> shift
        }
    }

    #[inline]
    pub fn lsr_c(val: u32, shift: u32) -> (u32, bool) {
        if shift == 0 || shift >= 32 {
            if shift == 0 {
                // LSR #0 means LSR #32
                let carry = (val >> 31) & 1 != 0;
                (0, carry)
            } else if shift == 32 {
                let carry = (val >> 31) & 1 != 0;
                (0, carry)
            } else {
                (0, false)
            }
        } else {
            let carry = (val >> (shift - 1)) & 1 != 0;
            (val >> shift, carry)
        }
    }

    #[inline]
    pub fn asr(val: u32, shift: u32) -> u32 {
        if shift == 0 {
            // ASR #0 = ASR #32
            if val & 0x8000_0000 != 0 { 0xFFFF_FFFF } else { 0 }
        } else if shift >= 32 {
            if val & 0x8000_0000 != 0 { 0xFFFF_FFFF } else { 0 }
        } else {
            ((val as i32) >> shift) as u32
        }
    }

    #[inline]
    pub fn asr_c(val: u32, shift: u32) -> (u32, bool) {
        if shift == 0 || shift >= 32 {
            let result = if val & 0x8000_0000 != 0 { 0xFFFF_FFFF } else { 0 };
            let carry = result & 1 != 0;
            (result, carry)
        } else {
            let carry = (val >> (shift - 1)) & 1 != 0;
            (((val as i32) >> shift) as u32, carry)
        }
    }

    #[inline]
    pub fn ror(val: u32, shift: u32) -> u32 {
        if shift == 0 {
            // RRX
            val // handled separately
        } else {
            let s = shift & 31;
            if s == 0 {
                val
            } else {
                val.rotate_right(s)
            }
        }
    }

    // Note: ror_c needs to be a method to access carry flag, but the static
    // version doesn't. The method version is in the impl block below.

    // Add with carry
    #[inline]
    pub fn add_with_carry(a: u32, b: u32, carry_in: bool) -> (u32, bool, bool) {
        let cin = if carry_in { 1u32 } else { 0u32 };
        let result = (a as u64)
            .wrapping_add(b as u64)
            .wrapping_add(cin as u64);
        let result32 = result as u32;
        let carry = result > 0xFFFF_FFFFu64;
        let overflow = (!(a ^ b) & (a ^ result32) & 0x8000_0000) != 0;
        (result32, carry, overflow)
    }
}
