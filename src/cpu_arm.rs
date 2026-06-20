// ARM instruction execution
use crate::cpu::*;
use crate::memory::Memory;
use core::cell::RefCell;

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
                
                // Check for PSR transfer (MRS/MSR): bits 27-20 = 0001 0rS0
                // MRS: 0001 0r00 (S=0), MSR: 0001 0r10 (S=1, bit21=1)
                // Important: bit 24 must be 0 (bit 24=1 means BX/misc)
                if bit24 == 0 && ((bits27_20 & 0x0FB) == 0x010 || (bits27_20 & 0x0FB) == 0x012) {
                    // PSR transfer (MRS or MSR)
                    self.exec_arm_psr_transfer(mem, instr);
                } else if (instr & 0x0190_F000) == 0x0100_F000 {
                    // BX / SWP / etc (misc) — bit 24=1 and bits 7-4=1111
                    self.exec_arm_misc(mem, instr);
                } else if (instr & 0x0000_00F0) == 0x0000_0090 {
                    // Multiply / swap (bit 7=1, bit 4=1)
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
                // Data processing - immediate (or PSR transfer)
                if (instr & 0x0190_F000) == 0x0120_F000 {
                    // MSR/MRS (PSR transfer)
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
            let field_mask = match (instr >> 16) & 0xF {
                0x1 => 0x0000_00FF,
                0x2 => 0x0000_FF00,
                0x4 => 0x00FF_0000,
                0x8 => 0xFF00_0000,
                0x3 => 0x0000_FFFF,
                0x5 => 0x00FF_00FF,
                0x9 => 0xFF00_00FF,
                0x6 => 0x00FF_FF00,
                0xA => 0xFFFF_0000,
                0x7 => 0x00FF_FFFF,
                0xB => 0xFFFF_00FF,
                0xC => 0x00FF_FFFF,
                0xD => 0xFF00_FFFF,
                0xE => 0xFFFF_FF00,
                0xF => 0xFFFF_FFFF,
                _ => 0,
            };

            if instr & 0x0040_0000 != 0 {
                // SPSR
                let spsr = self.get_spsr();
                self.set_spsr((spsr & !field_mask) | (val & field_mask));
            } else {
                // CPSR - control bits may change mode
                let old_mode = self.get_mode();
                let new_cpsr = (self.cpsr & !field_mask) | (val & field_mask);
                self.cpsr = new_cpsr;
                let new_mode = self.get_mode();
                if new_mode != old_mode {
                    self.switch_mode(new_mode);
                }
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
                let (r, c) = self.barrel_shift_result(op2, shift_carry);
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.r[15] = self.r[15].wrapping_add(4);
                self.cycles += 1;
                return;
            }
            0x9 => { // TEQ
                result = op1 ^ op2;
                let (r, c) = self.barrel_shift_result(op2, shift_carry);
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
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
                // In privileged mode, CPSR = SPSR
                let mode = self.get_mode();
                if mode != MODE_USR && mode != MODE_SYS {
                    self.cpsr = self.get_spsr();
                    let new_mode = self.get_mode();
                    if new_mode != mode {
                        self.switch_mode(new_mode);
                    }
                }
            } else {
                self.set_nz(result);
                self.set_flag(FLAG_C, shift_carry);
            }
        }

        // For arithmetic ops, S flag already handled
        if s && arith && rd == 15 {
            let mode = self.get_mode();
            if mode != MODE_USR && mode != MODE_SYS {
                self.cpsr = self.get_spsr();
                let new_mode = self.get_mode();
                if new_mode != mode {
                    self.switch_mode(new_mode);
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
                let mode = self.get_mode();
                if mode != MODE_USR && mode != MODE_SYS {
                    self.cpsr = self.get_spsr();
                    let new_mode = self.get_mode();
                    if new_mode != mode {
                        self.switch_mode(new_mode);
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

    fn exec_arm_swi(&mut self, _mem: &mut Memory, _instr: u32) {
        self.exception(EXC_SWI, MODE_SVC, true, false);
    }
}
