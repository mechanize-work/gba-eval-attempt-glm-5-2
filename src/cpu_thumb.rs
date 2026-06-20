// THUMB instruction execution
use crate::cpu::*;
use crate::memory::Memory;

impl Cpu {
    pub fn execute_thumb(&mut self, mem: &mut Memory, instr: u16) {
        let op = (instr >> 13) & 0x7;

        match op {
            0x0 => {
                // 000: Move shifted register
                self.exec_thumb_shift(instr);
            }
            0x1 => {
                // 001: Add/subtract
                self.exec_thumb_add_sub(instr);
            }
            0x2 => {
                // 010: Move/compare/add/subtract immediate
                self.exec_thumb_imm(instr);
            }
            0x3 => {
                // 011: ALU operations
                self.exec_thumb_alu(mem, instr);
            }
            0x4 => {
                // 100: Hi register operations / branch exchange
                self.exec_thumb_hi(mem, instr);
            }
            0x5 => {
                // 101: Load/store relative to PC
                self.exec_thumb_pc_rel(instr);
            }
            0x6 => {
                // 110: Load/store with register offset
                self.exec_thumb_reg_offset(instr);
            }
            0x7 => {
                // 111: Load/store sign-extended byte/halfword
                self.exec_thumb_signed_transfer(instr);
            }
            _ => unreachable!(),
        }

        // The 0x08-0x0F range (top 4 bits)
        let top4 = (instr >> 12) & 0xF;
        match top4 {
            0x8 => {
                // Load/store halfword
                self.exec_thumb_halfword_transfer(instr);
            }
            0x9 => {
                // SP-relative load/store
                self.exec_thumb_sp_rel(instr);
            }
            0xA => {
                // Load address
                self.exec_thumb_load_address(instr);
            }
            0xB => {
                // Add offset to stack pointer
                self.exec_thumb_add_sp(instr);
            }
            0xC => {
                // Push/pop registers
                self.exec_thumb_push_pop(instr);
            }
            0xD => {
                // Multiple load/store
                self.exec_thumb_multiple(instr);
            }
            0xE => {
                // Conditional branch
                self.exec_thumb_branch_cond(instr);
            }
            0xF => {
                // Software interrupt + Branches
                if (instr & 0xFF00) == 0xDF00 {
                    // SWI
                    self.exception(EXC_SWI, MODE_SVC, true, false);
                    return;
                } else if (instr & 0xF800) == 0xE000 {
                    // Unconditional branch
                    let offset = (instr & 0x7FF) as i32;
                    let offset = if offset & 0x400 != 0 {
                        offset | 0xFFFFF800
                    } else {
                        offset
                    };
                    let target = self.r[15].wrapping_add((offset as u32).wrapping_mul(2));
                    self.r[15] = target;
                    self.cycles += 3;
                    return;
                } else if (instr & 0xF000) == 0xF000 {
                    // Long branch with link (BL)
                    // This is a two-instruction sequence
                    // First: 1111 0 offset_high (push to LR)
                    // Second: 1111 1 offset_low (add to LR, branch)
                    self.exec_thumb_long_branch(instr);
                    return;
                }
            }
            _ => {}
        }
    }

    fn exec_thumb_shift(&mut self, instr: u16) {
        let op = (instr >> 11) & 0x3;
        let imm = (instr >> 6) & 0x1F;
        let rs = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let val = self.r[rs];
        let (result, carry) = match op {
            0 => { // LSL
                if imm == 0 {
                    (val, self.get_flag(FLAG_C))
                } else {
                    (val << imm, (val >> (32 - imm)) & 1 != 0)
                }
            }
            1 => { // LSR
                if imm == 0 {
                    // LSR #32
                    (0, (val >> 31) & 1 != 0)
                } else {
                    (val >> imm, (val >> (imm - 1)) & 1 != 0)
                }
            }
            2 => { // ASR
                if imm == 0 {
                    // ASR #32
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

        let operand = if is_imm {
            (instr >> 6) & 0x7
        } else {
            ((instr >> 6) & 0x7) as u16 as usize
        };

        let a = self.r[rs];
        let b = if is_imm {
            operand as u32
        } else {
            self.r[operand]
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
                // Multiply, signed carry is meaningless, N/Z set, C is undefined
                let result = a.wrapping_mul(b);
                self.r[rd] = result;
                self.set_nz(result);
                // C is unpredictable on ARMv4
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

        // R15 is allowed but uses current PC+4 (pipeline)
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
                return; // Don't add to PC again
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_pc_rel(&mut self, instr: u16) {
        let rd = ((instr >> 8) & 0x7) as usize;
        let imm = ((instr & 0xFF) as u32) << 2;
        // PC is aligned to 4
        let pc = self.r[15] & !2; // +4 for pipeline, aligned
        let val = pc.wrapping_add(4).wrapping_add(imm); // PC+4 + imm
        // Actually: PC is at instruction+4 in THUMB mode
        self.r[rd] = (self.r[15] & !3) + 4 + imm;
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 1;
    }

    fn exec_thumb_reg_offset(&mut self, mem: &mut Memory, instr: u16) {
        let op = (instr >> 10) & 0x3;
        let ro = ((instr >> 6) & 0x7) as usize;
        let rb = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(self.r[ro]);

        match op {
            0x0 => { // STR
                mem.write_word(addr, self.r[rd]);
            }
            0x1 => { // STRB
                mem.write_byte(addr, self.r[rd] as u8);
            }
            0x2 => { // LDR
                self.r[rd] = mem.read_word(addr & !3);
            }
            0x3 => { // LDRB
                self.r[rd] = mem.read_byte(addr) as u32;
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_signed_transfer(&mut self, mem: &mut Memory, instr: u16) {
        let op = (instr >> 10) & 0x3;
        let ro = ((instr >> 6) & 0x7) as usize;
        let rb = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(self.r[ro]);

        match op {
            0x0 => { // STRH
                mem.write_half(addr, self.r[rd] as u16);
            }
            0x1 => { // LDSB
                let val = mem.read_byte(addr) as i8 as u32;
                self.r[rd] = val;
            }
            0x2 => { // LDRH
                self.r[rd] = mem.read_half(addr) as u32;
            }
            0x3 => { // LDSH
                let val = mem.read_half(addr) as i16 as u32;
                self.r[rd] = val;
            }
            _ => {}
        }
        self.r[15] = self.r[15].wrapping_add(2);
        self.cycles += 2;
    }

    fn exec_thumb_halfword_transfer(&mut self, mem: &mut Memory, instr: u16) {
        let is_load = (instr >> 11) & 1 != 0;
        let imm = ((instr >> 6) & 0x1F) as u32 << 1;
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

    fn exec_thumb_sp_rel(&mut self, mem: &mut Memory, instr: u16) {
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
            // PC-relative: PC is instruction + 4, aligned to 4
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

    fn exec_thumb_push_pop(&mut self, mem: &mut Memory, instr: u16) {
        let is_load = (instr >> 11) & 1 != 0;
        let store_lr = (instr >> 8) & 1 != 0;
        let reg_list = instr & 0xFF;

        if is_load {
            // POP
            let count = reg_list.count_ones() as u32 + if store_lr { 1 } else { 0 };
            let mut addr = self.r[13];
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    self.r[i] = mem.read_word(addr);
                    addr = addr.wrapping_add(4);
                }
            }
            if store_lr {
                // Pop into R15 (PC)
                let val = mem.read_word(addr);
                self.r[15] = val & !1;
                addr = addr.wrapping_add(4);
            }
            self.r[13] = self.r[13].wrapping_add(count * 4);
            self.cycles += count as u64 + 2;
        } else {
            // PUSH
            let count = reg_list.count_ones() as u32 + if store_lr { 1 } else { 0 };
            let mut addr = self.r[13].wrapping_sub(count * 4);
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    mem.write_word(addr, self.r[i]);
                    addr = addr.wrapping_add(4);
                }
            }
            if store_lr {
                // Push R14 (LR)
                mem.write_word(addr, self.r[14]);
                addr = addr.wrapping_add(4);
            }
            self.r[13] = self.r[13].wrapping_sub(count * 4);
            self.cycles += count as u64 + 1;
        }
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_multiple(&mut self, mem: &mut Memory, instr: u16) {
        let is_load = (instr >> 11) & 1 != 0;
        let rb = ((instr >> 8) & 0x7) as usize;
        let reg_list = instr & 0xFF;

        if reg_list == 0 {
            // Undefined behavior - typically treat as R15, and adjust base by 0x40
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
            self.r[rb] = self.r[rb].wrapping_add(count * 4);
            self.cycles += count as u64 + 2;
        } else {
            // STMIA: write back happens regardless of whether Rb is in list
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    let val = if i == rb {
                        // Write-back value (original base + count*4)
                        self.r[rb].wrapping_add(count * 4)
                    } else {
                        self.r[i]
                    };
                    mem.write_word(addr, val);
                    addr = addr.wrapping_add(4);
                }
            }
            self.r[rb] = self.r[rb].wrapping_add(count * 4);
            self.cycles += count as u64 + 1;
        }
        self.r[15] = self.r[15].wrapping_add(2);
    }

    fn exec_thumb_branch_cond(&mut self, instr: u16) {
        let cond = (instr >> 8) & 0xF;
        if cond == 0xF {
            // This is actually a SWI in some encodings, but we handle that elsewhere
            self.r[15] = self.r[15].wrapping_add(2);
            return;
        }

        if self.check_cond(cond) {
            let offset = (instr & 0xFF) as i32;
            let offset = if offset & 0x80 != 0 {
                offset | 0xFFFFFF00
            } else {
                offset
            };
            let target = self.r[15].wrapping_add((offset as u32).wrapping_mul(2));
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
            // Low half: PC = LR + (offset << 1) + 4; LR = PC + 3 (next instr + 1)
            let offset_lo = offset << 1;
            let next_instr = self.r[15].wrapping_add(2);
            let target = self.r[14].wrapping_add(offset_lo);
            self.r[14] = next_instr | 1;
            self.r[15] = target & !1;
            self.cycles += 3;
        }
    }
}
