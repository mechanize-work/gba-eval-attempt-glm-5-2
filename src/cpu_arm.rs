// ARM instruction execution
use crate::cpu::*;
use crate::memory::Memory;
use core::cell::RefCell;

fn integer_sqrt(val: u32) -> u32 {
    if val == 0 { return 0; }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + val / x) / 2;
    }
    x
}

impl Cpu {
    pub fn execute_arm(&mut self, mem: &mut Memory, instr: u32) {
        let cond = (instr >> 28) & 0xF;
        if !self.check_cond(cond) {
            // Some instructions still need PC increment
            self.r[15] = self.r[15].wrapping_add(4);
            self.cycles += 1;
            return;
        }

        // Decode based on bit patterns
        let op = (instr >> 25) & 0x7;

        match op {
            0x0 => {
                // Bits 27-25 = 000: multiple instruction types
                let bits27_20 = (instr >> 20) & 0xFF;
                let bit24 = (instr >> 24) & 1;
                
                // Check for PSR transfer (MRS/MSR)
                // MRS: cond 0001 0r00 1111 rd 00000000 0000
                // MSR reg: cond 0001 0r10 1111 mask 0000 0000 Rm
                // Fixed bits: 0001 0xx0 1111 xxxx 0000 xxxx 0000
                // Mask: 0x0FB0_0FF0, value: 0x0100_0000 (covers both MRS and MSR-reg)
                // MRS/MSR with register: bits 27:24=0001, bit 23=0, bit 21 distinguishes MRS(0)/MSR(1), bits 11:8=0000, bits 7:4=0000
                // Mask out bit 22 (CPSR/SPSR select) and bits 19:12 (Rd/mask) 
                if (instr & 0x0F90_0FF0) == 0x0100_0000 {
                    // PSR transfer (MRS or MSR with register)
                    self.exec_arm_psr_transfer(mem, instr);
                } else if (instr & 0x0190_F000) == 0x0100_F000 {
                    // BX / SWP / etc (misc) — bit 24=1 and bits 7-4=1111
                    self.exec_arm_misc(mem, instr);
                } else if (instr & 0x0000_0090) == 0x0000_0090 && (instr & 0x0000_0060) != 0x0000_0000 {
                    // Halfword/signed load/store (bit 7=1, bit 4=1, bits 6:5 != 00)
                    self.exec_arm_halfword_transfer(mem, instr);
                } else if (instr & 0x0000_00F0) == 0x0000_0090 {
                    // Multiply / swap (bit 7=1, bit 4=1, bits 6:5 = 00)
                    self.exec_arm_multiply(mem, instr);
                } else if (instr & 0x0120_0000) == 0x0020_0000 {
                    // Immediate operand (bit 25=1... but op=0 means bit 25=0)
                    // This shouldn't happen, but handle it
                    self.exec_arm_data_processing(mem, instr, true);
                } else {
                    // Register operand with shift
                    self.exec_arm_data_processing(mem, instr, false);
                }
            }
            0x1 => {
                // Data processing - immediate (or MSR with immediate)
                // MSR immediate: cond 001 1 0 r 10 1111 rotate imm8
                // Check: bit 24=1, bit 23=0, bit 21=1, bit 20=0, bits 19:16=1111
                if (instr & 0x019F_0000) == 0x012F_0000 {
                    // MSR with immediate
                    self.exec_arm_psr_transfer(mem, instr);
                } else {
                    self.exec_arm_data_processing(mem, instr, true);
                }
            }
            0x2 | 0x3 => {
                // Load/store
                self.exec_arm_load_store(mem, instr);
            }
            0x4 => {
                // Load/store multiple
                self.exec_arm_load_store_multiple(mem, instr);
            }
            0x5 => {
                // Branch
                self.exec_arm_branch(mem, instr);
            }
            0x6 => {
                // Coprocessor load/store
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
            }
            0x7 => {
                // Coprocessor / SWI
                if (instr & 0x00FF_F000) == 0x00F0_0000 {
                    self.exec_arm_swi(mem, instr);
                } else {
                    self.r[15] = self.r[15].wrapping_add(4);
                    self.cycles += 1;
                }
            }
            _ => {
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
            }
        }
    }

    fn exec_arm_misc(&mut self, mem: &mut Memory, instr: u32) {
        let op = (instr >> 4) & 0xF;
        match op {
            0x1 => {
                // BX
                let rn = instr & 0xF;
                let addr = self.r[rn as usize];
                if addr & 1 != 0 {
                    self.cpsr |= FLAG_T;
                    self.r[15] = addr & !1;
                } else {
                    self.cpsr &= !FLAG_T;
                    self.r[15] = addr & !3;
                }
                self.cycles += 3;
            }
            0x3 => {
                // MRS (actually handled in psr_transfer, but this is for some edge cases)
                // Actually this shouldn't happen, MRS is 0x10F0
                self.r[15] = self.r[15].wrapping_add(4);
            }
            0x5 => {
                // QADD/QSUB etc (ARMv5TE)
                self.exec_arm_saturating(mem, instr);
            }
            0x7 => {
                // BKPT or undefined
                self.r[15] = self.r[15].wrapping_add(4);
            }
            0x8 | 0xA => {
                // SMLAxx etc (ARMv5TE multiply)
                self.exec_arm_mul_extra(mem, instr);
            }
            0x9 => {
                // SMLAW/SMULW
                self.exec_arm_mul_extra(mem, instr);
            }
            0xB => {
                // SMLALxx
                self.exec_arm_mul_extra(mem, instr);
            }
            0xC => {
                // SMULxx
                self.exec_arm_mul_extra(mem, instr);
            }
            0xD | 0xF => {
                // LDRD/STRD or undefined
                self.r[15] = self.r[15].wrapping_add(4);
            }
            _ => {
                self.r[15] = self.r[15].wrapping_add(4);
            }
        }
    }

    fn exec_arm_saturating(&mut self, _mem: &mut Memory, instr: u32) {
        let rd = ((instr >> 12) & 0xF) as usize;
        let rm = (instr & 0xF) as usize;
        let rn = ((instr >> 16) & 0xF) as usize;
        let op = (instr >> 21) & 0x3;

        let a = self.r[rn] as i32;
        let b = self.r[rm] as i32;

        fn _saturate(val: i64) -> i32 {
            if val > 0x7FFF_FFFF { 0x7FFF_FFFF }
            else if val < -0x8000_0000i64 { -0x8000_0000i32 }
            else { val as i32 }
        }

        let result = match op {
            0x0 => { // QADD
                let r = a.wrapping_add(b);
                if (r as i64) != (a as i64 + b as i64) {
                    self.cpsr |= FLAG_Q;
                }
                r
            }
            0x1 => { // QSUB
                let r = a.wrapping_sub(b);
                if (r as i64) != (a as i64 - b as i64) {
                    self.cpsr |= FLAG_Q;
                }
                r
            }
            0x2 => { // QDADD (double and add)
                let doubled = b.wrapping_add(b);
                if (doubled as i64) != (b as i64 * 2) {
                    self.cpsr |= FLAG_Q;
                }
                let r = a.wrapping_add(doubled);
                if (r as i64) != (a as i64 + doubled as i64) {
                    self.cpsr |= FLAG_Q;
                }
                r
            }
            0x3 => { // QDSUB
                let doubled = b.wrapping_add(b);
                if (doubled as i64) != (b as i64 * 2) {
                    self.cpsr |= FLAG_Q;
                }
                let r = a.wrapping_sub(doubled);
                if (r as i64) != (a as i64 - doubled as i64) {
                    self.cpsr |= FLAG_Q;
                }
                r
            }
            _ => 0,
        };
        self.r[rd] = result as u32;
        self.r[15] = self.r[15].wrapping_add(4);
        self.cycles += 1;
    }

    fn exec_arm_mul_extra(&mut self, _mem: &mut Memory, instr: u32) {
        // ARMv5TE extra multiply instructions
        // Simplified: just advance PC
        let rd = ((instr >> 16) & 0xF) as usize;
        let rs = ((instr >> 12) & 0xF) as usize;
        let rm = ((instr >> 8) & 0xF) as usize;
        let rn = (instr & 0xF) as usize;

        let m = self.r[rm] as i16 as i32;
        let s = self.r[rs] as i16 as i32;

        match (instr >> 21) & 0xF {
            0x0 => { // SMLABB etc: rd = rn + (rm[15:0] * rs[15:0])
                let prod = (m as i64 * s as i64) as i32;
                let result = self.r[rn] as i32;
                let sum = result.wrapping_add(prod);
                if (sum as i64) != (result as i64 + prod as i64) {
                    self.cpsr |= FLAG_Q;
                }
                self.r[rd] = sum as u32;
            }
            0x1 => { // SMLATB
                let prod = (m as i64 * (self.r[rs] >> 16) as i16 as i32 as i64) as i32;
                let result = self.r[rn] as i32;
                let sum = result.wrapping_add(prod);
                if (sum as i64) != (result as i64 + prod as i64) {
                    self.cpsr |= FLAG_Q;
                }
                self.r[rd] = sum as u32;
            }
            0x2 => { // SMLABT
                let prod = ((self.r[rm] >> 16) as i16 as i32 as i64 * s as i64) as i32;
                let result = self.r[rn] as i32;
                let sum = result.wrapping_add(prod);
                if (sum as i64) != (result as i64 + prod as i64) {
                    self.cpsr |= FLAG_Q;
                }
                self.r[rd] = sum as u32;
            }
            0x3 => { // SMLATT
                let prod = ((self.r[rm] >> 16) as i16 as i32 as i64 * (self.r[rs] >> 16) as i16 as i32 as i64) as i32;
                let result = self.r[rn] as i32;
                let sum = result.wrapping_add(prod);
                if (sum as i64) != (result as i64 + prod as i64) {
                    self.cpsr |= FLAG_Q;
                }
                self.r[rd] = sum as u32;
            }
            0x8 => { // SMULBB
                self.r[rd] = (m as i64 * s as i64) as u32;
            }
            0x9 => { // SMULTB
                self.r[rd] = (m as i64 * (self.r[rs] >> 16) as i16 as i32 as i64) as u32;
            }
            0xA => { // SMULBT
                self.r[rd] = ((self.r[rm] >> 16) as i16 as i32 as i64 * s as i64) as u32;
            }
            0xB => { // SMULTT
                self.r[rd] = ((self.r[rm] >> 16) as i16 as i32 as i64 * (self.r[rs] >> 16) as i16 as i32 as i64) as u32;
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(4);
        self.cycles += 1;
    }

    fn exec_arm_psr_transfer(&mut self, _mem: &mut Memory, instr: u32) {
        let is_mrs = (instr & 0x0020_0000) == 0;

        if is_mrs {
            // MRS rd, CPSR/SPSR
            let rd = ((instr >> 12) & 0xF) as usize;
            if instr & 0x0040_0000 != 0 {
                self.r[rd] = self.get_spsr();
            } else {
                self.r[rd] = self.cpsr;
            }
        } else {
            // MSR CPSR/SPSR, <val>
            let val = if (instr & 0x0200_0000) != 0 {
                // Immediate
                let imm = instr & 0xFF;
                let ror = ((instr >> 8) & 0xF) * 2;
                if ror == 0 { imm } else { imm.rotate_right(ror) }
            } else {
                // Register
                let rm = (instr & 0xF) as usize;
                self.r[rm]
            };

            // Mask
            // Field mask per ARM spec:
            // bit 19 (f) -> 0xF0000000 (NZCV flags)
            // bit 18 (s) -> 0x0F000000 (status)
            // bit 17 (x) -> 0x00FF0000 (extension)
            // bit 16 (c) -> 0x0000FFFF (control)
            let field = (instr >> 16) & 0xF;
            let field_mask = {
                let mut m = 0u32;
                if field & 0x8 != 0 { m |= 0xF000_0000; } // f
                if field & 0x4 != 0 { m |= 0x0F00_0000; } // s
                if field & 0x2 != 0 { m |= 0x00FF_0000; } // x
                if field & 0x1 != 0 { m |= 0x0000_FFFF; } // c
                m
            };

            if instr & 0x0040_0000 != 0 {
                // SPSR
                let spsr = self.get_spsr();
                self.set_spsr((spsr & !field_mask) | (val & field_mask));
            } else {
                // CPSR - control bits may change mode
                let old_mode = self.get_mode();
                let new_cpsr = (self.cpsr & !field_mask) | (val & field_mask);
                let new_mode = new_cpsr & 0x1F;
                if new_mode != old_mode {
                    // Switch mode BEFORE updating CPSR so banking works
                    self.switch_mode_from(old_mode, new_mode);
                }
                self.cpsr = new_cpsr;
                // Check if we need to switch ARM/THUMB
                // This is handled by the execution loop
            }
        }
        self.r[15] = self.r[15].wrapping_add(4);
        self.cycles += 1;
    }

    fn exec_arm_data_processing(&mut self, mem: &mut Memory, instr: u32, immediate: bool) {
        let opcode = (instr >> 21) & 0xF;
        let s = (instr >> 20) & 1 != 0;
        let rn = ((instr >> 16) & 0xF) as usize;
        let rd = ((instr >> 12) & 0xF) as usize;

        // Get operand1
        let op1 = if rn == 15 {
            self.r[15] + 8 // PC is 2 instructions ahead in ARM
        } else {
            self.r[rn]
        };

        // Get operand2 and shift carry
        let (op2, shift_carry) = if immediate {
            let imm = instr & 0xFF;
            let ror = ((instr >> 8) & 0xF) * 2;
            let val = if ror == 0 { imm } else { imm.rotate_right(ror) };
            let carry = if ror == 0 {
                self.get_flag(FLAG_C)
            } else {
                (val >> 31) & 1 != 0
            };
            (val, carry)
        } else {
            let rm = (instr & 0xF) as usize;
            let shift_amount = if (instr >> 4) & 1 != 0 {
                // Register shift
                let rs = ((instr >> 8) & 0xF) as usize;
                self.r[rs] & 0xFF
            } else {
                // Immediate shift
                (instr >> 7) & 0x1F
            };

            let rm_val = if rm == 15 {
                self.r[15] + 8
            } else {
                self.r[rm]
            };

            let shift_type = (instr >> 5) & 0x3;
            self.barrel_shift(rm_val, shift_type, shift_amount, (instr >> 4) & 1 != 0)
        };

        let result: u32;
        let mut arith = false;

        match opcode {
            0x0 => { // AND
                result = op1 & op2;
            }
            0x1 => { // EOR
                result = op1 ^ op2;
            }
            0x2 => { // SUB
                let (r, c, v) = Cpu::add_with_carry(op1, !op2, true);
                result = r;
                if s { self.set_nzcv(result, c, v); }
                arith = true;
            }
            0x3 => { // RSB
                let (r, c, v) = Cpu::add_with_carry(!op1, op2, true);
                result = r;
                if s { self.set_nzcv(result, c, v); }
                arith = true;
            }
            0x4 => { // ADD
                let (r, c, v) = Cpu::add_with_carry(op1, op2, false);
                result = r;
                if s { self.set_nzcv(result, c, v); }
                arith = true;
            }
            0x5 => { // ADC
                let cin = self.get_flag(FLAG_C);
                let (r, c, v) = Cpu::add_with_carry(op1, op2, cin);
                result = r;
                if s { self.set_nzcv(result, c, v); }
                arith = true;
            }
            0x6 => { // SBC
                let cin = self.get_flag(FLAG_C);
                let (r, c, v) = Cpu::add_with_carry(op1, !op2, cin);
                result = r;
                if s { self.set_nzcv(result, c, v); }
                arith = true;
            }
            0x7 => { // RSC
                let cin = self.get_flag(FLAG_C);
                let (r, c, v) = Cpu::add_with_carry(!op1, op2, cin);
                result = r;
                if s { self.set_nzcv(result, c, v); }
                arith = true;
            }
            0x8 => { // TST
                result = op1 & op2;
                self.set_nz(result);
                self.set_flag(FLAG_C, shift_carry);
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
                return;
            }
            0x9 => { // TEQ
                result = op1 ^ op2;
                self.set_nz(result);
                self.set_flag(FLAG_C, shift_carry);
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
                return;
            }
            0xA => { // CMP
                let (r, c, v) = Cpu::add_with_carry(op1, !op2, true);
                result = r;
                self.set_nzcv(result, c, v);
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
                return;
            }
            0xB => { // CMN
                let (r, c, v) = Cpu::add_with_carry(op1, op2, false);
                result = r;
                self.set_nzcv(result, c, v);
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
                return;
            }
            0xC => { // ORR
                result = op1 | op2;
            }
            0xD => { // MOV
                result = op2;
            }
            0xE => { // BIC
                result = op1 & !op2;
            }
            0xF => { // MVN
                result = !op2;
            }
            _ => { result = 0; }
        }

        // Write result
        if rd == 15 {
            self.r[15] = result & !1;
            if result & 1 != 0 {
                self.cpsr |= FLAG_T;
                self.r[15] &= !1;
            } else {
                self.r[15] &= !3;
            }
            self.cycles += 3;
        } else {
            self.r[rd] = result;
            self.r[15] = self.r[15].wrapping_add(4);
            self.cycles += 1;
        }

        // Set flags for logical ops
        if s && !arith {
            if rd == 15 {
                let old_mode = self.get_mode();
                if old_mode != MODE_USR && old_mode != MODE_SYS {
                    self.cpsr = self.get_spsr();
                    let new_mode = self.get_mode();
                    if new_mode != old_mode {
                        self.switch_mode_from(old_mode, new_mode);
                    }
                }
            } else {
                self.set_nz(result);
                self.set_flag(FLAG_C, shift_carry);
            }
        }

        // For arithmetic ops, S flag already handled
        if s && arith && rd == 15 {
            let old_mode = self.get_mode();
            if old_mode != MODE_USR && old_mode != MODE_SYS {
                self.cpsr = self.get_spsr();
                let new_mode = self.get_mode();
                if new_mode != old_mode {
                    self.switch_mode_from(old_mode, new_mode);
                }
            }
        }
    }

    // Barrel shifter that returns (result, carry)
    fn barrel_shift(&self, val: u32, shift_type: u32, amount: u32, register_shift: bool) -> (u32, bool) {
        match shift_type {
            0 => { // LSL
                if register_shift {
                    if amount == 0 {
                        (val, self.get_flag(FLAG_C))
                    } else if amount < 32 {
                        (val << amount, (val >> (32 - amount)) & 1 != 0)
                    } else if amount == 32 {
                        (0, val & 1 != 0)
                    } else {
                        (0, false)
                    }
                } else {
                    if amount == 0 {
                        (val, self.get_flag(FLAG_C))
                    } else {
                        (val << amount, (val >> (32 - amount)) & 1 != 0)
                    }
                }
            }
            1 => { // LSR
                let amt = if !register_shift && amount == 0 { 32 } else { amount };
                if amt == 0 {
                    (val, self.get_flag(FLAG_C))
                } else if amt < 32 {
                    (val >> amt, (val >> (amt - 1)) & 1 != 0)
                } else if amt == 32 {
                    (0, (val >> 31) & 1 != 0)
                } else {
                    (0, false)
                }
            }
            2 => { // ASR
                let amt = if !register_shift && amount == 0 { 32 } else { amount };
                if amt == 0 {
                    (val, self.get_flag(FLAG_C))
                } else if amt < 32 {
                    let carry = (val >> (amt - 1)) & 1 != 0;
                    (((val as i32) >> amt) as u32, carry)
                } else {
                    if val & 0x8000_0000 != 0 {
                        (0xFFFF_FFFF, true)
                    } else {
                        (0, false)
                    }
                }
            }
            3 => { // ROR/RRX
                if register_shift {
                    if amount == 0 {
                        (val, self.get_flag(FLAG_C))
                    } else {
                        let amt = amount & 31;
                        if amt == 0 {
                            (val, (val >> 31) & 1 != 0)
                        } else {
                            (val.rotate_right(amt), (val >> (amt - 1)) & 1 != 0)
                        }
                    }
                } else {
                    if amount == 0 {
                        // RRX
                        let carry = val & 1 != 0;
                        let result = (val >> 1) | (if self.get_flag(FLAG_C) { 0x8000_0000 } else { 0 });
                        (result, carry)
                    } else {
                        (val.rotate_right(amount), (val >> (amount - 1)) & 1 != 0)
                    }
                }
            }
            _ => (val, self.get_flag(FLAG_C))
        }
    }

    fn barrel_shift_result(&self, val: u32, carry: bool) -> (u32, bool) {
        (val, carry)
    }

    fn exec_arm_load_store(&mut self, mem: &mut Memory, instr: u32) {
        let is_load = (instr >> 20) & 1 != 0;
        let write_back = (instr >> 21) & 1 != 0;
        let byte = (instr >> 22) & 1 != 0;
        let up = (instr >> 23) & 1 != 0;
        let pre = (instr >> 24) & 1 != 0;
        let rn = ((instr >> 16) & 0xF) as usize;
        let rd = ((instr >> 12) & 0xF) as usize;

        // Compute offset
        let offset = if (instr >> 25) & 1 != 0 {
            // Register/immediate shifted
            let rm = (instr & 0xF) as usize;
            let shift_type = (instr >> 5) & 0x3;
            let shift_amount = (instr >> 7) & 0x1F;
            let rm_val = if rm == 15 { self.r[15] + 8 } else { self.r[rm] };
            let (shifted, _) = self.barrel_shift(rm_val, shift_type, shift_amount, false);
            shifted
        } else {
            instr & 0xFFF
        };

        let base = if rn == 15 {
            self.r[15] + 8
        } else {
            self.r[rn]
        };

        let addr = if up {
            base.wrapping_add(offset)
        } else {
            base.wrapping_sub(offset)
        };

        let effective_addr = if pre { addr } else { base };

        if is_load {
            let val = if byte {
                mem.read_byte(effective_addr) as u32
            } else {
                mem.read_word(effective_addr & !3)
            };
            if rd == 15 {
                self.r[15] = val & !1;
                if val & 1 != 0 {
                    self.cpsr |= FLAG_T;
                } else {
                    self.r[15] &= !3;
                }
                self.cycles += 3;
            } else {
                self.r[rd] = val;
                self.cycles += 2;
            }
        } else {
            let val = if rd == 15 {
                self.r[15] + 12
            } else {
                self.r[rd]
            };
            if byte {
                mem.write_byte(effective_addr, val as u8);
            } else {
                mem.write_word(effective_addr & !3, val);
            }
            self.cycles += 2;
        }

        // Write-back
        if !pre || write_back {
            if !is_load || rd != rn {
                self.r[rn] = addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
    }

    fn exec_arm_halfword_transfer(&mut self, mem: &mut Memory, instr: u32) {
        // LDRH/STRH/LDRSH/LDRSB
        // Encoding: cond 000PUBWL Rn Rd offset 1SH1
        // bits 7:4 determine type:
        //   1011 = LDRH (unsigned halfword)
        //   1101 = LDRSB (signed byte)
        //   1110 = STRH
        //   1111 = LDRSH (signed halfword)
        let is_load = (instr >> 20) & 1 != 0;
        let write_back = (instr >> 21) & 1 != 0;
        let up = (instr >> 23) & 1 != 0;
        let pre = (instr >> 24) & 1 != 0;
        let rn = ((instr >> 16) & 0xF) as usize;
        let rd = ((instr >> 12) & 0xF) as usize;
        let sh = (instr >> 5) & 0x3; // bits 6:5

        // Compute offset
        let offset = if (instr >> 22) & 1 != 0 {
            // Immediate offset: split across bits 11:8 and 3:0
            (((instr >> 8) & 0xF) << 4) | (instr & 0xF)
        } else {
            // Register offset
            let rm = (instr & 0xF) as usize;
            if rm == 15 { self.r[15] + 8 } else { self.r[rm] }
        };

        let base = if rn == 15 { self.r[15] + 8 } else { self.r[rn] };

        let addr = if up {
            base.wrapping_add(offset)
        } else {
            base.wrapping_sub(offset)
        };

        let effective_addr = if pre { addr } else { base };

        if is_load {
            let val: u32;
            match sh {
                0b01 => { // LDRH (unsigned halfword)
                    val = mem.read_half(effective_addr) as u32;
                }
                0b10 => { // LDRSB (signed byte)
                    let b = mem.read_byte(effective_addr) as i8 as i32 as u32;
                    val = b;
                }
                0b11 => { // LDRSH (signed halfword)
                    let h = mem.read_half(effective_addr) as i16 as i32 as u32;
                    val = h;
                }
                _ => { val = 0; }
            }
            if rd == 15 {
                self.r[15] = val & !1;
                if val & 1 != 0 {
                    self.cpsr |= FLAG_T;
                } else {
                    self.r[15] &= !3;
                }
                self.cycles += 3;
            } else {
                self.r[rd] = val;
                self.cycles += 2;
            }
        } else {
            // STRH
            let val = if rd == 15 { self.r[15] + 12 } else { self.r[rd] };
            mem.write_half(effective_addr, val as u16);
            self.cycles += 2;
        }

        // Write-back
        if !pre || write_back {
            if !is_load || rd != rn {
                self.r[rn] = addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
    }

    fn exec_arm_load_store_multiple(&mut self, mem: &mut Memory, instr: u32) {
        let is_load = (instr >> 20) & 1 != 0;
        let write_back = (instr >> 21) & 1 != 0;
        let s_bit = (instr >> 22) & 1 != 0; // User bank / force SPSR
        let up = (instr >> 23) & 1 != 0;
        let pre = (instr >> 24) & 1 != 0;
        let rn = ((instr >> 16) & 0xF) as usize;
        let reg_list = instr & 0xFFFF;

        if reg_list == 0 {
            // Empty register list: loads/stores R15, and RN = RN ± 0x40
            // This is an edge case, let's handle it
            self.r[15] = self.r[15].wrapping_add(4);
            self.cycles += 1;
            return;
        }

        let count = reg_list.count_ones() as u32;

        // Compute base address
        let mut base = self.r[rn];
        let end = if up {
            base.wrapping_add(4 * count)
        } else {
            base.wrapping_sub(4 * count)
        };

        let mut addr = if up {
            if pre { base.wrapping_add(4) } else { base }
        } else {
            if pre { base.wrapping_sub(4) } else { base.wrapping_sub(4 * count) }
        };

        // For descending, we iterate from low to high but addresses go down
        // Actually let's restructure: compute start address properly
        let start_addr = if up {
            if pre { base.wrapping_add(4) } else { base }
        } else {
            if pre { end.wrapping_sub(4) } else { end }
        };

        // Wait, let me redo this more carefully.
        // The registers are stored/loaded from lowest to highest register number.
        // For IA (up, post=0): addr starts at base, increments
        // For IB (up, pre=1): addr starts at base+4, increments
        // For DA (down, pre=0): addr starts at base, decrements (but we access from end)
        // For DB (down, pre=1): addr starts at base-4, decrements

        // Let's just iterate registers 0-15 and assign addresses
        let mut cur_addr = if up {
            if pre { base.wrapping_add(4) } else { base }
        } else {
            // For descending: the lowest reg gets the lowest address
            // which is base - 4*count + 4 (pre) or base - 4*count (post)
            if pre { base.wrapping_sub(4 * count) } else { base.wrapping_sub(4 * count) }
        };

        // Hmm, this is getting complicated. Let me just do it the simple way:
        // For ascending: iterate regs 0-15, each gets successive addresses
        // For descending: iterate regs 15-0, but addresses go from high to low

        let mut addresses = [0u32; 16];
        let mut idx = 0;
        if up {
            let mut a = if pre { base.wrapping_add(4) } else { base };
            for i in 0..16 {
                if reg_list & (1 << i) != 0 {
                    addresses[i] = a;
                    a = a.wrapping_add(4);
                    idx += 1;
                }
            }
        } else {
            let mut a = if pre { base.wrapping_sub(4) } else { base };
            for i in (0..16).rev() {
                if reg_list & (1 << i) != 0 {
                    addresses[i] = a;
                    a = a.wrapping_sub(4);
                    idx += 1;
                }
            }
        }

        let new_base = if up {
            base.wrapping_add(4 * count)
        } else {
            base.wrapping_sub(4 * count)
        };

        if is_load {
            let mode = self.get_mode();
            let force_user = s_bit && mode != MODE_USR && mode != MODE_SYS;

            for i in 0..16 {
                if reg_list & (1 << i) != 0 {
                    let val = mem.read_word(addresses[i]);
                    if i == 15 {
                        self.r[15] = val & !1;
                        if val & 1 != 0 {
                            self.cpsr |= FLAG_T;
                        } else {
                            self.r[15] &= !3;
                        }
                    } else if force_user {
                        // Load into user bank registers (simplified)
                        self.r[i] = val;
                    } else {
                        self.r[i] = val;
                    }
                }
            }

            // S bit with PC load: CPSR = SPSR
            if s_bit && (reg_list & 0x8000) != 0 {
                let old_mode = self.get_mode();
                if old_mode != MODE_USR && old_mode != MODE_SYS {
                    self.cpsr = self.get_spsr();
                    let new_mode = self.get_mode();
                    if new_mode != old_mode {
                        self.switch_mode_from(old_mode, new_mode);
                    }
                }
            }

            self.cycles += count as u64 + 2;
        } else {
            let mode = self.get_mode();
            let force_user = s_bit && mode != MODE_USR && mode != MODE_SYS;

            for i in 0..16 {
                if reg_list & (1 << i) != 0 {
                    let val = if i == 15 {
                        self.r[15] + 12
                    } else if force_user {
                        // Use user bank registers (simplified - just use current)
                        self.r[i]
                    } else {
                        self.r[i]
                    };
                    mem.write_word(addresses[i], val);
                }
            }
            self.cycles += count as u64 + 1;
        }

        // Write-back
        if write_back {
            self.r[rn] = new_base;
        }

        self.r[15] = self.r[15].wrapping_add(4);
    }

    fn exec_arm_branch(&mut self, _mem: &mut Memory, instr: u32) {
        let link = (instr >> 24) & 1 != 0;
        let offset = (instr & 0x00FF_FFFF) as i32;
        // Sign extend
        let offset = if offset & 0x0080_0000 != 0 {
            offset | (0xFF00_0000u32 as i32)
        } else {
            offset
        };
        let offset = (offset as i64) * 4;

        // ARM pipeline: PC = current instruction + 8
        let pc = self.r[15].wrapping_add(8);

        if link {
            self.r[14] = self.r[15].wrapping_add(4); // LR = next instruction
        }

        let target = pc.wrapping_add(offset as u32);
        self.r[15] = target;
        self.cycles += 3;
    }

    fn exec_arm_multiply(&mut self, _mem: &mut Memory, instr: u32) {
        // Multiply instructions: MUL, MLA, UMULL, UMLAL, SMULL, SMLAL
        // SWP, SWPB
        let bit24 = (instr >> 24) & 1;
        let bit23 = (instr >> 23) & 1;
        let bit22 = (instr >> 22) & 1; // SWP: B bit
        let bit21 = (instr >> 21) & 1; // Accumulate
        let bit20 = (instr >> 20) & 1; // Set flags

        if bit24 == 0 && bit23 == 0 {
            // MUL or MLA
            let rd = ((instr >> 16) & 0xF) as usize;
            let rs = ((instr >> 8) & 0xF) as usize;
            let rm = (instr & 0xF) as usize;

            let result = self.r[rm].wrapping_mul(self.r[rs]);
            if bit21 != 0 {
                // MLA: Rd = Rm * Rs + Rn
                let rn = ((instr >> 12) & 0xF) as usize;
                self.r[rd] = result.wrapping_add(self.r[rn]);
            } else {
                // MUL: Rd = Rm * Rs
                self.r[rd] = result;
            }

            if bit20 != 0 {
                self.set_nz(self.r[rd]);
                // C is unpredictable on ARMv4
            }
            self.r[15] = self.r[15].wrapping_add(4);
            self.cycles += 2;
        } else {
            // UMULL, UMLAL, SMULL, SMLAL (64-bit multiply)
            let rd_hi = ((instr >> 16) & 0xF) as usize;
            let rd_lo = ((instr >> 12) & 0xF) as usize;
            let rs = ((instr >> 8) & 0xF) as usize;
            let rm = (instr & 0xF) as usize;

            if bit23 != 0 {
                // Signed multiply
                let a = self.r[rm] as i32 as i64;
                let b = self.r[rs] as i32 as i64;
                let result = a.wrapping_mul(b);

                if bit21 != 0 {
                    // SMLAL: RdHiRdLo = Rm * Rs + RdHiRdLo
                    let acc = ((self.r[rd_hi] as u64) << 32) | (self.r[rd_lo] as u64);
                    let final_result = result.wrapping_add(acc as i64);
                    self.r[rd_hi] = (final_result >> 32) as u32;
                    self.r[rd_lo] = final_result as u32;
                } else {
                    // SMULL: RdHiRdLo = Rm * Rs
                    self.r[rd_hi] = (result >> 32) as u32;
                    self.r[rd_lo] = result as u32;
                }
            } else {
                // Unsigned multiply
                let a = self.r[rm] as u64;
                let b = self.r[rs] as u64;
                let result = a.wrapping_mul(b);

                if bit21 != 0 {
                    // UMLAL: RdHiRdLo = Rm * Rs + RdHiRdLo
                    let acc = ((self.r[rd_hi] as u64) << 32) | (self.r[rd_lo] as u64);
                    let final_result = result.wrapping_add(acc);
                    self.r[rd_hi] = (final_result >> 32) as u32;
                    self.r[rd_lo] = final_result as u32;
                } else {
                    // UMULL: RdHiRdLo = Rm * Rs
                    self.r[rd_hi] = (result >> 32) as u32;
                    self.r[rd_lo] = result as u32;
                }
            }

            if bit20 != 0 {
                self.set_nz(self.r[rd_hi]);
            }
            self.r[15] = self.r[15].wrapping_add(4);
            self.cycles += 3;
        }
    }

    fn exec_arm_swi(&mut self, mem: &mut Memory, instr: u32) {
        // ARM SWI: number is in bits 23-0
        // But on GBA, only bits 7-0 matter (24-bit comment field, lower 8 = function)
        let swi_num = (instr >> 0) & 0xFF;
        self.do_swi(mem, swi_num);
    }

    /// Implement GBA BIOS SWI functions directly instead of running the BIOS stub handler.
    /// Only intercept SWI calls from ROM code (0x08000000+), not from BIOS itself.
    pub fn do_swi(&mut self, mem: &mut Memory, swi_num: u32) {
        // If SWI is called from BIOS (PC < 0x08000000), use the BIOS stub handler
        let caller_pc = self.r[15];
        if caller_pc < 0x0800_0000 {
            // BIOS internal SWI - use exception handler
            self.exception(EXC_SWI, MODE_SVC, true, false);
            return;
        }

        // Game SWI - intercept and implement directly
        // Determine PC increment based on current mode
        let pc_inc: u32 = if self.is_thumb() { 2 } else { 4 };

        match swi_num {
            0x00 => {
                // SoftReset - jump to ROM entry
                // R0 selects: 0=clear EWRAM, 1=don't clear
                // Just jump to 0x08000000
                self.r[15] = 0x08000000;
                self.cpsr &= !FLAG_T;
                self.cycles += 1;
            }
            0x01 => {
                // RegRamReset - reset registers and RAM
                // R0 = mask: bit 0=EWRAM, 1=IWRAM, 2=palette, 3=VRAM, 4=OAM
                // bit 5=IO registers (except DISPCNT etc), 6=sound registers
                // bit 7=other registers
                // For simplicity, just advance PC
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
            0x02 => {
                // Halt - halt CPU until interrupt
                self.halted = true;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
            0x03 => {
                // Stop (Stop/Sleep)
                self.halted = true;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
            0x04 => {
                // IntrWait - wait for interrupt
                // R0: 0=wait for any, 1=wait for specific (R1=IE, R2=IF)
                self.halted = true;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
            0x05 => {
                // VBlankIntrWait - wait for NEXT VBlank
                // Clear VBlank IF, halt CPU, wait for VBlank to wake
                eprintln!("SWI VBlankIntrWait called from PC={:08X}", self.r[15]);
                let if_val = mem.read_half(0x0400_0202);
                mem.write_half(0x0400_0202, if_val & !1);
                self.halted = true;
                self.vblank_intr_wait = true;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
            0x04 => {
                // IntrWait - wait for interrupt
                self.halted = true;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
            0x06 => {
                // Div: R0 / R1 -> R0=quotient, R1=remainder, R3=abs(quotient)
                let a = self.r[0] as i32;
                let b = self.r[1] as i32;
                if b != 0 {
                    self.r[0] = (a / b) as u32;
                    self.r[1] = (a % b) as u32;
                    self.r[3] = (a / b).abs() as u32;
                }
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 10;
            }
            0x07 => {
                // DivArm: R1 / R0 (reversed operand order)
                let a = self.r[1] as i32;
                let b = self.r[0] as i32;
                if b != 0 {
                    self.r[0] = (a / b) as u32;
                    self.r[1] = (a % b) as u32;
                    self.r[3] = (a / b).abs() as u32;
                }
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 10;
            }
            0x08 => {
                // Sqrt: R0 -> R0
                let val = self.r[0];
                self.r[0] = integer_sqrt(val);
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 10;
            }
            0x0B => {
                // CpuSet: copy from R0 to R1, R2 = count (lower 20 bits) | options
                // R2 bit 24: 0=16-bit, 1=32-bit
                // R2 bit 26: 0=increment, 1=fixed source
                let src = self.r[0];
                let dst = self.r[1];
                let count = self.r[2] & 0x1FFFFF;
                let is_32bit = self.r[2] & 0x0400_0000 != 0;
                let fixed_src = self.r[2] & 0x0800_0000 != 0;

                if is_32bit {
                    let mut s = src;
                    let mut d = dst;
                    for _ in 0..count {
                        let val = mem.read_word(s);
                        mem.write_word(d, val);
                        d = d.wrapping_add(4);
                        if !fixed_src { s = s.wrapping_add(4); }
                    }
                } else {
                    let mut s = src;
                    let mut d = dst;
                    for _ in 0..count {
                        let val = mem.read_half(s);
                        mem.write_half(d, val);
                        d = d.wrapping_add(2);
                        if !fixed_src { s = s.wrapping_add(2); }
                    }
                }
                self.r[0] = src.wrapping_add(if fixed_src { 0 } else { if is_32bit { count * 4 } else { count * 2 } });
                self.r[1] = dst.wrapping_add(if is_32bit { count * 4 } else { count * 2 });
                self.r[2] = 0;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += count as u64 + 5;
            }
            0x0C => {
                // CpuFastSet: fast copy from R0 to R1, R2 = count (in words)
                // Always 32-bit, copies in chunks of 8 words (32 bytes)
                let src = self.r[0] & !3; // Word-align source
                let dst = self.r[1] & !3; // Word-align dest
                let mut count = self.r[2] & 0x0003_FFFF; // lower 22 bits
                let mut s = src;
                let mut d = dst;

                // CpuFastSet copies in chunks of 8 words (32 bytes)
                while count >= 8 {
                    for _ in 0..8 {
                        let val = mem.read_word(s);
                        mem.write_word(d, val);
                        s = s.wrapping_add(4);
                        d = d.wrapping_add(4);
                    }
                    count -= 8;
                }
                // Handle remaining words
                for _ in 0..count {
                    let val = mem.read_word(s);
                    mem.write_word(d, val);
                    s = s.wrapping_add(4);
                    d = d.wrapping_add(4);
                }
                self.r[0] = src;
                self.r[1] = dst;
                self.r[2] = 0;
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 5;
            }
            _ => {
                // Unknown SWI - just advance PC
                self.r[15] = self.r[15].wrapping_add(pc_inc);
                self.cycles += 1;
            }
        }
    }
}
