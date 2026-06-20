// THUMB instruction execution
use crate::cpu::*;
use crate::memory::Memory;

impl Cpu {
    pub fn execute_thumb(&mut self, mem: &mut Memory, instr: u16) {
        // THUMB instruction decoding uses bits 15-10 for primary dispatch
        let op = (instr >> 10) & 0x3F;

        match op {
            0x00..=0x01 => {
                // THUMB.1: LSL
                self.exec_thumb_shift(instr, 0);
            }
            0x02..=0x03 => {
                // THUMB.1: LSR
                self.exec_thumb_shift(instr, 1);
            }
            0x04..=0x05 => {
                // THUMB.1: ASR
                self.exec_thumb_shift(instr, 2);
            }
            0x06..=0x07 => {
                // THUMB.2: add/subtract
                self.exec_thumb_add_sub(instr);
            }
            0x08..=0x0F => {
                // THUMB.3: Move/compare/add/subtract immediate
                self.exec_thumb_imm(instr);
            }
            0x10 => {
                // THUMB.4: ALU operations
                self.exec_thumb_alu(mem, instr);
            }
            0x11 => {
                // THUMB.5: Hi register operations / branch exchange
                self.exec_thumb_hi(mem, instr);
            }
            0x12..=0x13 => {
                // THUMB.6: Load from PC-relative address
                self.exec_thumb_pc_rel(instr);
            }
            0x14..=0x17 => {
                // THUMB.7: Load/store with register offset
                self.exec_thumb_reg_offset(instr, mem);
            }
            0x18..=0x1B => {
                // THUMB.8: Load/store sign-extended byte/halfword with register offset
                self.exec_thumb_signed_transfer(instr, mem);
            }
            0x1C..=0x1F => {
                // THUMB.9: Load/store with immediate offset (word/byte)
                self.exec_thumb_imm_offset(instr, mem);
            }
            0x20..=0x21 => {
                // THUMB.10: Load/store halfword with immediate offset
                self.exec_thumb_halfword_imm_offset(instr, mem);
            }
            0x22..=0x23 | 0x26..=0x27 | 0x2A..=0x2B | 0x32..=0x33 | 0x3A..=0x3B => {
                // Undefined/reserved
                self.r[15] = self.r[15].wrapping_add(2);
                self.cycles += 1;
            }
            0x24..=0x25 => {
                // THUMB.11: SP-relative load/store
                self.exec_thumb_sp_rel(instr, mem);
            }
            0x28..=0x29 => {
                // THUMB.12: Load address (PC or SP relative)
                self.exec_thumb_load_address(instr);
            }
            0x2C..=0x2D => {
                // THUMB.13: Add offset to stack pointer
                self.exec_thumb_add_sp(instr);
            }
            0x2E => {
                // THUMB.14: Push registers
                self.exec_thumb_push(mem, instr);
            }
            0x2F => {
                // THUMB.14: Pop registers
                self.exec_thumb_pop(mem, instr);
            }
            0x30..=0x33 => {
                // THUMB.15: Multiple load/store (STMIA/LDMIA)
                // bit 11 = L (0=store, 1=load), bits 10-8 = Rb
                self.exec_thumb_multiple(instr, mem);
            }
            0x34..=0x37 => {
                // THUMB.16: Conditional branch
                self.exec_thumb_branch_cond(instr);
            }
            0x38..=0x39 => {
                // THUMB.17: Unconditional branch
                let offset = (instr & 0x7FF) as i32;
                let offset = if offset & 0x400 != 0 {
                    offset | (0xFFFFF800u32 as i32)
                } else {
                    offset
                };
                // PC = PC + 4 + offset*2 (pipeline: PC is current+4 in thumb)
                let target = self.r[15].wrapping_add(4).wrapping_add((offset as u32).wrapping_mul(2));
                self.r[15] = target;
                self.cycles += 3;
            }
            0x3C..=0x3D => {
                // THUMB.18: Long branch with link (first instruction)
                self.exec_thumb_long_branch(instr);
            }
            0x3E..=0x3F => {
                // THUMB.18: Long branch with link (second instruction)
                self.exec_thumb_long_branch(instr);
            }
            _ => {
                // SWI: 0xDF00-0xDFFF is handled by bits 15-8 = 0xDF
                if (instr & 0xFF00) == 0xDF00 {
                    self.exception(EXC_SWI, MODE_SVC, true, false);
                    return;
                }
                self.r[15] = self.r[15].wrapping_add(2);
                self.cycles += 1;
            }
        }
    }

    fn exec_thumb_shift(&mut self, instr: u16, shift_type: u16) {
        let imm = (instr >> 6) & 0x1F;
        let rs = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let val = self.r[rs];
        let (result, carry) = match shift_type {
            0 => { // LSL
                if imm == 0 {
                    (val, self.get_flag(FLAG_C))
                } else {
                    (val << imm, (val >> (32 - imm)) & 1 != 0)
                }
            }
            1 => { // LSR
                if imm == 0 {
                    (0, (val >> 31) & 1 != 0)
                } else {
                    (val >> imm, (val >> (imm - 1)) & 1 != 0)
                }
            }
            2 => { // ASR
                if imm == 0 {
                    if val & 0x8000_0000 != 0 {
                        (0xFFFF_FFFF, true)
                    } else {
                        (0, false)
                    }
                } else {
                    let carry = (val >> (imm - 1)) & 1 != 0;
                    (((val as i32) >> imm) as u32, carry)
                }
            }
            _ => (val, self.get_flag(FLAG_C))
        };

        self.r[rd] = result;
        self.set_nz(result);
        self.set_flag(FLAG_C, carry);
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_add_sub(&mut self, instr: u16) {
        let op = (instr >> 9) & 0x3;
        let is_imm = (instr >> 10) & 1 != 0;
        let rs = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let operand_idx = ((instr >> 6) & 0x7) as usize;

        let a = self.r[rs];
        let b = if is_imm {
            operand_idx as u32
        } else {
            self.r[operand_idx]
        };

        let (result, carry, overflow) = match op {
            0 => { // ADD
                Cpu::add_with_carry(a, b, false)
            }
            1 => { // SUB
                Cpu::add_with_carry(a, !b, true)
            }
            2 => { // ADD with carry
                let cin = self.get_flag(FLAG_C);
                Cpu::add_with_carry(a, b, cin)
            }
            3 => { // SUB with carry
                let cin = self.get_flag(FLAG_C);
                Cpu::add_with_carry(a, !b, cin)
            }
            _ => (0, false, false)
        };

        self.r[rd] = result;
        self.set_nzcv(result, carry, overflow);
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_imm(&mut self, instr: u16) {
        let op = (instr >> 11) & 0x3;
        let rd = ((instr >> 8) & 0x7) as usize;
        let imm = (instr & 0xFF) as u32;

        match op {
            0 => { // MOV
                self.r[rd] = imm;
                self.set_nz(imm);
            }
            1 => { // CMP
                let (result, carry, overflow) = Cpu::add_with_carry(self.r[rd], !imm, true);
                self.set_nzcv(result, carry, overflow);
            }
            2 => { // ADD
                let (result, carry, overflow) = Cpu::add_with_carry(self.r[rd], imm, false);
                self.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            3 => { // SUB
                let (result, carry, overflow) = Cpu::add_with_carry(self.r[rd], !imm, true);
                self.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_alu(&mut self, _mem: &mut Memory, instr: u16) {
        let op = (instr >> 6) & 0xF;
        let rs = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let a = self.r[rd];
        let b = self.r[rs];

        match op {
            0x0 => { // AND
                let result = a & b;
                self.r[rd] = result;
                self.set_nz(result);
            }
            0x1 => { // EOR
                let result = a ^ b;
                self.r[rd] = result;
                self.set_nz(result);
            }
            0x2 => { // LSL
                let amount = b & 0xFF;
                let (result, carry) = if amount == 0 {
                    (a, self.get_flag(FLAG_C))
                } else if amount < 32 {
                    (a << amount, (a >> (32 - amount)) & 1 != 0)
                } else if amount == 32 {
                    (0, a & 1 != 0)
                } else {
                    (0, false)
                };
                self.r[rd] = result;
                self.set_nz(result);
                self.set_flag(FLAG_C, carry);
            }
            0x3 => { // LSR
                let amount = b & 0xFF;
                let (result, carry) = if amount == 0 {
                    (a, self.get_flag(FLAG_C))
                } else if amount < 32 {
                    (a >> amount, (a >> (amount - 1)) & 1 != 0)
                } else if amount == 32 {
                    (0, (a >> 31) & 1 != 0)
                } else {
                    (0, false)
                };
                self.r[rd] = result;
                self.set_nz(result);
                self.set_flag(FLAG_C, carry);
            }
            0x4 => { // ASR
                let amount = b & 0xFF;
                let (result, carry) = if amount == 0 {
                    (a, self.get_flag(FLAG_C))
                } else if amount < 32 {
                    (((a as i32) >> amount) as u32, (a >> (amount - 1)) & 1 != 0)
                } else {
                    if a & 0x8000_0000 != 0 {
                        (0xFFFF_FFFF, true)
                    } else {
                        (0, false)
                    }
                };
                self.r[rd] = result;
                self.set_nz(result);
                self.set_flag(FLAG_C, carry);
            }
            0x5 => { // ADC
                let cin = self.get_flag(FLAG_C);
                let (result, carry, overflow) = Cpu::add_with_carry(a, b, cin);
                self.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            0x6 => { // SBC
                let cin = self.get_flag(FLAG_C);
                let (result, carry, overflow) = Cpu::add_with_carry(a, !b, cin);
                self.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            0x7 => { // ROR
                let amount = b & 0xFF;
                let (result, carry) = if amount == 0 {
                    (a, self.get_flag(FLAG_C))
                } else {
                    let amt = amount & 31;
                    if amt == 0 {
                        (a, (a >> 31) & 1 != 0)
                    } else {
                        (a.rotate_right(amt), (a >> (amt - 1)) & 1 != 0)
                    }
                };
                self.r[rd] = result;
                self.set_nz(result);
                self.set_flag(FLAG_C, carry);
            }
            0x8 => { // TST
                let result = a & b;
                self.set_nz(result);
            }
            0x9 => { // NEG
                let (result, carry, overflow) = Cpu::add_with_carry(0, !b, true);
                self.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            0xA => { // CMP
                let (result, carry, overflow) = Cpu::add_with_carry(a, !b, true);
                self.set_nzcv(result, carry, overflow);
            }
            0xB => { // CMN
                let (result, carry, overflow) = Cpu::add_with_carry(a, b, false);
                self.set_nzcv(result, carry, overflow);
            }
            0xC => { // ORR
                let result = a | b;
                self.r[rd] = result;
                self.set_nz(result);
            }
            0xD => { // MUL
                let result = a.wrapping_mul(b);
                self.r[rd] = result;
                self.set_nz(result);
            }
            0xE => { // BIC
                let result = a & !b;
                self.r[rd] = result;
                self.set_nz(result);
            }
            0xF => { // MVN
                let result = !b;
                self.r[rd] = result;
                self.set_nz(result);
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_hi(&mut self, _mem: &mut Memory, instr: u16) {
        let op = (instr >> 8) & 0x3;
        let h1 = (instr >> 7) & 1;
        let h2 = (instr >> 6) & 1;
        let rs = (((instr >> 3) & 0x7) | (h2 << 3)) as usize;
        let rd = ((instr & 0x7) | (h1 << 3)) as usize;

        let get_reg = |cpu: &Cpu, idx: usize| -> u32 {
            if idx == 15 {
                cpu.r[15] + 4
            } else {
                cpu.r[idx]
            }
        };

        match op {
            0x0 => { // ADD
                let b = get_reg(self, rs);
                let result = self.r[rd].wrapping_add(b);
                self.r[rd] = result;
                if rd == 15 {
                    self.r[15] &= !1;
                    self.cycles += 3;
                } else {
                    self.cycles += 1;
                }
            }
            0x1 => { // CMP
                let a = get_reg(self, rd);
                let b = get_reg(self, rs);
                let (result, carry, overflow) = Cpu::add_with_carry(a, !b, true);
                self.set_nzcv(result, carry, overflow);
                self.cycles += 1;
            }
            0x2 => { // MOV
                let b = get_reg(self, rs);
                self.r[rd] = b;
                if rd == 15 {
                    self.r[15] &= !1;
                    self.cycles += 3;
                } else {
                    self.cycles += 1;
                }
            }
            0x3 => { // BX
                let b = get_reg(self, rs);
                if b & 1 != 0 {
                    self.cpsr |= FLAG_T;
                    self.r[15] = b & !1;
                } else {
                    self.cpsr &= !FLAG_T;
                    self.r[15] = b & !3;
                }
                self.cycles += 3;
                return;
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_pc_rel(&mut self, instr: u16) {
        let rd = ((instr >> 8) & 0x7) as usize;
        let imm = ((instr & 0xFF) as u32) << 2;
        // PC is current instruction + 4, aligned to 4
        self.r[rd] = (self.r[15] & !3).wrapping_add(4).wrapping_add(imm);
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_reg_offset(&mut self, instr: u16, mem: &mut Memory) {
        let op = (instr >> 10) & 0x3;
        let ro = ((instr >> 6) & 0x7) as usize;
        let rb = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(self.r[ro]);

        match op {
            0x0 => { mem.write_word(addr, self.r[rd]); }
            0x1 => { mem.write_byte(addr, self.r[rd] as u8); }
            0x2 => { self.r[rd] = mem.read_word(addr & !3); }
            0x3 => { self.r[rd] = mem.read_byte(addr) as u32; }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_signed_transfer(&mut self, instr: u16, mem: &mut Memory) {
        // THUMB.8: Load/store sign-extended byte/halfword
        // Also includes STRH and LDRH
        let op = (instr >> 10) & 0x3;
        let ro = ((instr >> 6) & 0x7) as usize;
        let rb = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(self.r[ro]);

        // H flag (bit 11) and S flag (bit 10) determine the operation:
        // 00 = STRH, 01 = LDSB, 10 = LDRH, 11 = LDSH
        match op {
            0x0 => { mem.write_half(addr, self.r[rd] as u16); }
            0x1 => { self.r[rd] = mem.read_byte(addr) as i8 as u32; } // LDSB
            0x2 => { self.r[rd] = mem.read_half(addr) as u32; }       // LDRH
            0x3 => { self.r[rd] = mem.read_half(addr) as i16 as u32; } // LDSH
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_imm_offset(&mut self, instr: u16, mem: &mut Memory) {
        // THUMB.9: Load/store with immediate offset
        // Bit 12: 0=byte, 1=word
        let is_word = (instr >> 12) & 1 != 0;
        let is_load = (instr >> 11) & 1 != 0;
        let offset_val = if is_word {
            ((instr & 0x7FF) as u32) << 2  // word offset
        } else {
            (instr & 0x7FF) as u32  // byte offset
        };
        let rb = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(offset_val);

        if is_word {
            if is_load {
                self.r[rd] = mem.read_word(addr & !3);
            } else {
                mem.write_word(addr & !3, self.r[rd]);
            }
        } else {
            if is_load {
                self.r[rd] = mem.read_byte(addr) as u32;
            } else {
                mem.write_byte(addr, self.r[rd] as u8);
            }
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_halfword_imm_offset(&mut self, instr: u16, mem: &mut Memory) {
        // THUMB.10: Load/store halfword with immediate offset
        let is_load = (instr >> 11) & 1 != 0;
        let imm = ((instr & 0x7FF) as u32) << 1;
        let rb = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(imm);

        if is_load {
            self.r[rd] = mem.read_half(addr) as u32;
        } else {
            mem.write_half(addr, self.r[rd] as u16);
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_sp_rel(&mut self, instr: u16, mem: &mut Memory) {
        let is_load = (instr >> 11) & 1 != 0;
        let rd = ((instr >> 8) & 0x7) as usize;
        let imm = ((instr & 0xFF) as u32) << 2;
        let addr = self.r[13].wrapping_add(imm);

        if is_load {
            self.r[rd] = mem.read_word(addr);
        } else {
            mem.write_word(addr, self.r[rd]);
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_load_address(&mut self, instr: u16) {
        let is_sp = (instr >> 11) & 1 != 0;
        let rd = ((instr >> 8) & 0x7) as usize;
        let imm = ((instr & 0xFF) as u32) << 2;

        if is_sp {
            self.r[rd] = self.r[13].wrapping_add(imm);
        } else {
            self.r[rd] = (self.r[15] & !3).wrapping_add(4).wrapping_add(imm);
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_add_sp(&mut self, instr: u16) {
        let is_sub = (instr >> 7) & 1 != 0;
        let imm = ((instr & 0x7F) as u32) << 2;
        if is_sub {
            self.r[13] = self.r[13].wrapping_sub(imm);
        } else {
            self.r[13] = self.r[13].wrapping_add(imm);
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_push(&mut self, mem: &mut Memory, instr: u16) {
        let store_lr = (instr >> 8) & 1 != 0;
        let reg_list = instr & 0xFF;

        let count = reg_list.count_ones() as u32 + if store_lr { 1 } else { 0 };
        let mut addr = self.r[13].wrapping_sub(count * 4);
        for i in 0..8 {
            if reg_list & (1 << i) != 0 {
                mem.write_word(addr, self.r[i]);
                addr = addr.wrapping_add(4);
            }
        }
        if store_lr {
            mem.write_word(addr, self.r[14]);
            addr = addr.wrapping_add(4);
        }
        self.r[13] = self.r[13].wrapping_sub(count * 4);
        self.cycles += count as u64 + 1;
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_pop(&mut self, mem: &mut Memory, instr: u16) {
        let load_pc = (instr >> 8) & 1 != 0;
        let reg_list = instr & 0xFF;

        let count = reg_list.count_ones() as u32 + if load_pc { 1 } else { 0 };
        let mut addr = self.r[13];
        for i in 0..8 {
            if reg_list & (1 << i) != 0 {
                self.r[i] = mem.read_word(addr);
                addr = addr.wrapping_add(4);
            }
        }
        if load_pc {
            let val = mem.read_word(addr);
            self.r[15] = val & !1;
            addr = addr.wrapping_add(4);
            self.cycles += 3;
        } else {
            self.cycles += 1;
        }
        self.r[13] = self.r[13].wrapping_add(count * 4);
        self.cycles += count as u64 + 1;
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_multiple(&mut self, instr: u16, mem: &mut Memory) {
        let is_load = (instr >> 11) & 1 != 0;
        let rb = ((instr >> 8) & 0x7) as usize;
        let reg_list = instr & 0xFF;

        if reg_list == 0 {
            self.r[rb] = self.r[rb].wrapping_add(0x40);
            self.r[15] = self.r[15].wrapping_add(2);
            return;
        }

        let count = reg_list.count_ones() as u32;
        let mut addr = self.r[rb];

        if is_load {
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    self.r[i] = mem.read_word(addr);
                    addr = addr.wrapping_add(4);
                }
            }
            self.cycles += count as u64 + 2;
        } else {
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    let val = if i == rb {
                        self.r[rb].wrapping_add(count * 4)
                    } else {
                        self.r[i]
                    };
                    mem.write_word(addr, val);
                    addr = addr.wrapping_add(4);
                }
            }
            self.cycles += count as u64 + 1;
        }
        self.r[rb] = self.r[rb].wrapping_add(count * 4);
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_branch_cond(&mut self, instr: u16) {
        let cond = ((instr >> 8) & 0xF) as u32;
        if cond == 0xF {
            // SWI
            self.exception(EXC_SWI, MODE_SVC, true, false);
            return;
        }

        if self.check_cond(cond) {
            let offset = (instr & 0xFF) as i32;
            let offset = if offset & 0x80 != 0 {
                offset | (0xFFFFFF00u32 as i32)
            } else {
                offset
            };
            // PC + 4 pipeline offset for THUMB
            let target = self.r[15].wrapping_add(4).wrapping_add((offset as u32).wrapping_mul(2));
            self.r[15] = target;
            self.cycles += 3;
        } else {
            self.r[15] = self.r[15].wrapping_add(2);
            self.cycles += 1;
        }
    }

    fn exec_thumb_long_branch(&mut self, instr: u16) {
        let is_low = (instr >> 11) & 1 != 0;
        let offset = (instr & 0x7FF) as u32;

        if !is_low {
            // High half: LR = PC + (offset << 12) + 4
            let offset_hi = if offset & 0x400 != 0 {
                (offset | 0xFFFFF800) << 12
            } else {
                offset << 12
            };
            self.r[14] = self.r[15].wrapping_add(offset_hi).wrapping_add(4);
            self.r[15] = self.r[15].wrapping_add(2);
            self.cycles += 1;
        } else {
            // Low half: PC = LR + (offset << 1); LR = next_instr + 1
            let offset_lo = offset << 1;
            let next_instr = self.r[15].wrapping_add(2);
            let target = self.r[14].wrapping_add(offset_lo);
            self.r[14] = next_instr | 1;
            self.r[15] = target & !1;
            self.cycles += 3;
        }
    }
}
