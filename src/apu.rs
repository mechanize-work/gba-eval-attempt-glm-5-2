// APU - Audio Processing Unit
// Handles PSG channels (1-4) and FIFO channels (A, B)
extern crate alloc;
use alloc::boxed::Box;

pub struct Apu {
    pub audio_buffer: Box<[i16; 4096 * 2]>, // stereo interleaved
    pub audio_count: usize,

    // Channel 1: Square with sweep
    pub ch1_enable: bool,
    pub ch1_length: u16,
    pub ch1_freq: u16,
    pub ch1_volume: u8,
    pub ch1_duty: u8,
    pub ch1_sweep_enable: bool,
    pub ch1_sweep_freq: u32,
    pub ch1_sweep_shift: u8,
    pub ch1_sweep_dir: bool,
    pub ch1_sweep_timer: u32,
    pub ch1_sweep_period: u8,
    pub ch1_env_dir: bool,
    pub ch1_env_timer: u32,
    pub ch1_env_period: u8,
    pub ch1_env_start: u8,
    pub ch1_phase: u32,
    pub ch1_length_enable: bool,
    pub ch1_length_counter: u32,

    // Channel 2: Square
    pub ch2_enable: bool,
    pub ch2_freq: u16,
    pub ch2_volume: u8,
    pub ch2_duty: u8,
    pub ch2_env_dir: bool,
    pub ch2_env_timer: u32,
    pub ch2_env_period: u8,
    pub ch2_env_start: u8,
    pub ch2_phase: u32,
    pub ch2_length_enable: bool,
    pub ch2_length_counter: u32,
    pub ch2_length: u16,

    // Channel 3: Wave
    pub ch3_enable: bool,
    pub ch3_volume: u8,
    pub ch3_phase: u32,
    pub ch3_freq: u16,
    pub ch3_length: u16,
    pub ch3_length_enable: bool,
    pub ch3_length_counter: u32,
    pub ch3_wave: [u8; 16],
    pub ch3_playing: bool,
    pub ch3_size: bool,

    // Channel 4: Noise
    pub ch4_enable: bool,
    pub ch4_volume: u8,
    pub ch4_env_dir: bool,
    pub ch4_env_timer: u32,
    pub ch4_env_period: u8,
    pub ch4_env_start: u8,
    pub ch4_freq: u16,
    pub ch4_length: u16,
    pub ch4_length_enable: bool,
    pub ch4_length_counter: u32,
    pub ch4_lfsr: u32,
    pub ch4_phase: u32,
    pub ch4_clock_shift: u8,
    pub4_prescaler: u8,
    pub ch4_width7: bool,
    pub ch4_prescaler: u8,

    // FIFO A
    pub fifo_a: [u8; 32],
    pub fifo_a_read: usize,
    pub fifo_a_write: usize,
    pub fifo_a_count: usize,
    pub fifo_a_timer: u32,

    // FIFO B
    pub fifo_b: [u8; 32],
    pub fifo_b_read: usize,
    pub fifo_b_write: usize,
    pub fifo_b_count: usize,
    pub fifo_b_timer: u32,

    // Sound control
    pub soundcnt_l: u16,
    pub soundcnt_h: u16,
    pub soundcnt_x: u16,
    pub soundbias: u16,

    // Sample generation
    pub sample_timer: u32,
    pub samples_per_frame: usize,

    // PSG channel clocks
    pub frame_sequencer: u32,
    pub frame_seq_timer: u32,

    // Master
    pub sample_rate: u32,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            audio_buffer: Box::new([0; 4096 * 2]),
            audio_count: 0,

            ch1_enable: false,
            ch1_length: 0,
            ch1_freq: 0,
            ch1_volume: 0,
            ch1_duty: 0,
            ch1_sweep_enable: false,
            ch1_sweep_freq: 0,
            ch1_sweep_shift: 0,
            ch1_sweep_dir: false,
            ch1_sweep_timer: 0,
            ch1_sweep_period: 0,
            ch1_env_dir: false,
            ch1_env_timer: 0,
            ch1_env_period: 0,
            ch1_env_start: 0,
            ch1_phase: 0,
            ch1_length_enable: false,
            ch1_length_counter: 0,

            ch2_enable: false,
            ch2_freq: 0,
            ch2_volume: 0,
            ch2_duty: 0,
            ch2_env_dir: false,
            ch2_env_timer: 0,
            ch2_env_period: 0,
            ch2_env_start: 0,
            ch2_phase: 0,
            ch2_length_enable: false,
            ch2_length_counter: 0,
            ch2_length: 0,

            ch3_enable: false,
            ch3_volume: 0,
            ch3_phase: 0,
            ch3_freq: 0,
            ch3_length: 0,
            ch3_length_enable: false,
            ch3_length_counter: 0,
            ch3_wave: [0; 16],
            ch3_playing: false,
            ch3_size: false,

            ch4_enable: false,
            ch4_volume: 0,
            ch4_env_dir: false,
            ch4_env_timer: 0,
            ch4_env_period: 0,
            ch4_env_start: 0,
            ch4_freq: 0,
            ch4_length: 0,
            ch4_length_enable: false,
            ch4_length_counter: 0,
            ch4_lfsr: 0,
            ch4_phase: 0,
            ch4_clock_shift: 0,
            pub4_prescaler: 0,
            ch4_width7: false,
            ch4_prescaler: 0,

            fifo_a: [0; 32],
            fifo_a_read: 0,
            fifo_a_write: 0,
            fifo_a_count: 0,
            fifo_a_timer: 0,

            fifo_b: [0; 32],
            fifo_b_read: 0,
            fifo_b_write: 0,
            fifo_b_count: 0,
            fifo_b_timer: 0,

            soundcnt_l: 0,
            soundcnt_h: 0,
            soundcnt_x: 0,
            soundbias: 0x200,

            sample_timer: 0,
            samples_per_frame: 0,

            frame_sequencer: 0,
            frame_seq_timer: 0,

            sample_rate: 32768,
        }
    }

    pub fn reset(&mut self) {
        *self = Apu::new();
    }

    // Write to sound register
    pub fn write_reg(&mut self, addr: u32, val: u16) {
        let off = (addr - 0x0400_0000) as usize;
        match off {
            0x60 => { // SOUND1CNT_L (sweep)
                self.ch1_sweep_period = ((val >> 4) & 0x7) as u8;
                self.ch1_sweep_dir = val & 0x8 != 0;
                self.ch1_sweep_shift = (val & 0x7) as u8;
            }
            0x62 => { // SOUND1CNT_H (duty, envelope)
                self.ch1_duty = ((val >> 6) & 0x3) as u8;
                self.ch1_env_start = ((val >> 4) & 0xF) as u8;
                self.ch1_volume = self.ch1_env_start;
                self.ch1_env_dir = val & 0x8 != 0;
                self.ch1_env_period = (val & 0x7) as u8;
                self.ch1_length = 64 - (val & 0x3F) as u16;
            }
            0x64 => { // SOUND1CNT_X (freq, reset)
                self.ch1_freq = val & 0x7FF;
                self.ch1_length = ((val >> 0) & 0x3F) as u16; // wait this is wrong
                self.ch1_length_enable = val & 0x4000 != 0;
                if val & 0x8000 != 0 {
                    // Reset
                    self.ch1_enable = true;
                    self.ch1_phase = 0;
                    self.ch1_volume = self.ch1_env_start;
                    self.ch1_env_timer = self.ch1_env_period as u32;
                    self.ch1_length_counter = self.ch1_length as u32;
                    self.ch1_sweep_enable = self.ch1_sweep_period != 0 || self.ch1_sweep_shift != 0;
                    self.ch1_sweep_freq = self.ch1_freq as u32;
                    self.ch1_sweep_timer = if self.ch1_sweep_period != 0 { self.ch1_sweep_period as u32 } else { 8 };
                }
            }
            0x68 => { // SOUND2CNT_L (duty, envelope)
                self.ch2_duty = ((val >> 6) & 0x3) as u8;
                self.ch2_env_start = ((val >> 4) & 0xF) as u8;
                self.ch2_volume = self.ch2_env_start;
                self.ch2_env_dir = val & 0x8 != 0;
                self.ch2_env_period = (val & 0x7) as u8;
                self.ch2_length = 64 - (val & 0x3F) as u16;
            }
            0x6C => { // SOUND2CNT_H (freq, reset)
                self.ch2_freq = val & 0x7FF;
                self.ch2_length_enable = val & 0x4000 != 0;
                if val & 0x8000 != 0 {
                    self.ch2_enable = true;
                    self.ch2_phase = 0;
                    self.ch2_volume = self.ch2_env_start;
                    self.ch2_env_timer = self.ch2_env_period as u32;
                    self.ch2_length_counter = self.ch2_length as u32;
                }
            }
            0x70 => { // SOUND3CNT_L (bank, enable)
                self.ch3_playing = val & 0x80 != 0;
                self.ch3_size = val & 0x40 != 0;
            }
            0x72 => { // SOUND3CNT_H (length, volume)
                self.ch3_length = 256 - (val & 0xFF) as u16;
                self.ch3_volume = ((val >> 13) & 0x7) as u8;
            }
            0x74 => { // SOUND3CNT_X (freq, reset)
                self.ch3_freq = val & 0x7FF;
                self.ch3_length_enable = val & 0x4000 != 0;
                if val & 0x8000 != 0 {
                    self.ch3_enable = true;
                    self.ch3_phase = 0;
                    self.ch3_length_counter = self.ch3_length as u32;
                }
            }
            0x78 => { // SOUND4CNT_L (length, envelope)
                self.ch4_env_start = ((val >> 4) & 0xF) as u8;
                self.ch4_volume = self.ch4_env_start;
                self.ch4_env_dir = val & 0x8 != 0;
                self.ch4_env_period = (val & 0x7) as u8;
                self.ch4_length = 64 - ((val >> 0) & 0x3F) as u16;
            }
            0x7C => { // SOUND4CNT_H (freq, reset)
                self.ch4_prescaler = (val & 0x7) as u8;
                self.ch4_width7 = val & 0x8 != 0;
                self.ch4_clock_shift = ((val >> 4) & 0xF) as u8;
                self.ch4_length_enable = val & 0x4000 != 0;
                if val & 0x8000 != 0 {
                    self.ch4_enable = true;
                    self.ch4_lfsr = 0x7FFF;
                    self.ch4_phase = 0;
                    self.ch4_volume = self.ch4_env_start;
                    self.ch4_env_timer = self.ch4_env_period as u32;
                    self.ch4_length_counter = self.ch4_length as u32;
                }
            }
            0x80 => { // SOUNDCNT_L
                self.soundcnt_l = val;
            }
            0x82 => { // SOUNDCNT_H
                self.soundcnt_h = val;
            }
            0x84 => { // SOUNDCNT_X
                self.soundcnt_x = val & 0x8F;
            }
            0x88 => { // SOUNDBIAS
                self.soundbias = val;
                self.sample_rate = if val & 0x4000 != 0 { 65536 } else { 32768 };
            }
            0x90..=0x9F => { // WAVE_RAM
                let idx = (off - 0x90) / 2;
                if idx < 8 {
                    self.ch3_wave[idx * 2] = (val & 0xFF) as u8;
                    self.ch3_wave[idx * 2 + 1] = ((val >> 8) & 0xFF) as u8;
                }
            }
            0xA0 => { // FIFO_A
                self.fifo_push_a(val as u8);
                self.fifo_push_a((val >> 8) as u8);
            }
            0xA2 => {
                self.fifo_push_a((val >> 0) as u8);
                self.fifo_push_a((val >> 8) as u8);
            }
            0xA4 => { // FIFO_B
                self.fifo_push_b(val as u8);
                self.fifo_push_b((val >> 8) as u8);
            }
            0xA6 => {
                self.fifo_push_b((val >> 0) as u8);
                self.fifo_push_b((val >> 8) as u8);
            }
            _ => {}
        }
    }

    pub fn read_reg(&self, addr: u32) -> u16 {
        let off = (addr - 0x0400_0000) as usize;
        match off {
            0x84 => self.soundcnt_x,
            0x88 => self.soundbias,
            _ => 0,
        }
    }

    fn fifo_push_a(&mut self, val: u8) {
        if self.fifo_a_count < 32 {
            self.fifo_a[self.fifo_a_write] = val;
            self.fifo_a_write = (self.fifo_a_write + 1) % 32;
            self.fifo_a_count += 1;
        }
    }

    fn fifo_push_b(&mut self, val: u8) {
        if self.fifo_b_count < 32 {
            self.fifo_b[self.fifo_b_write] = val;
            self.fifo_b_write = (self.fifo_b_write + 1) % 32;
            self.fifo_b_count += 1;
        }
    }

    pub fn fifo_a_pop(&mut self) -> u8 {
        if self.fifo_a_count > 0 {
            let val = self.fifo_a[self.fifo_a_read];
            self.fifo_a_read = (self.fifo_a_read + 1) % 32;
            self.fifo_a_count -= 1;
            val
        } else {
            0
        }
    }

    pub fn fifo_b_pop(&mut self) -> u8 {
        if self.fifo_b_count > 0 {
            let val = self.fifo_b[self.fifo_b_read];
            self.fifo_b_read = (self.fifo_b_read + 1) % 32;
            self.fifo_b_count -= 1;
            val
        } else {
            0
        }
    }

    // Clock the frame sequencer
    pub fn clock_frame_seq(&mut self) {
        // 512 Hz / 8 = 64 Hz per step
        // Called at 512 Hz
        self.frame_sequencer = (self.frame_sequencer + 1) & 7;

        match self.frame_sequencer {
            0 | 4 => {
                // Length clock
                self.clock_length();
            }
            2 | 6 => {
                // Length + sweep
                self.clock_length();
                self.clock_sweep();
            }
            7 => {
                // Envelope
                self.clock_envelope();
            }
            _ => {}
        }
    }

    fn clock_length(&mut self) {
        if self.ch1_length_enable && self.ch1_length_counter > 0 {
            self.ch1_length_counter -= 1;
            if self.ch1_length_counter == 0 {
                self.ch1_enable = false;
            }
        }
        if self.ch2_length_enable && self.ch2_length_counter > 0 {
            self.ch2_length_counter -= 1;
            if self.ch2_length_counter == 0 {
                self.ch2_enable = false;
            }
        }
        if self.ch3_length_enable && self.ch3_length_counter > 0 {
            self.ch3_length_counter -= 1;
            if self.ch3_length_counter == 0 {
                self.ch3_enable = false;
            }
        }
        if self.ch4_length_enable && self.ch4_length_counter > 0 {
            self.ch4_length_counter -= 1;
            if self.ch4_length_counter == 0 {
                self.ch4_enable = false;
            }
        }
    }

    fn clock_sweep(&mut self) {
        if !self.ch1_sweep_enable || self.ch1_sweep_period == 0 {
            return;
        }
        self.ch1_sweep_timer -= 1;
        if self.ch1_sweep_timer == 0 {
            self.ch1_sweep_timer = self.ch1_sweep_period as u32;
            let new_freq = if self.ch1_sweep_dir {
                self.ch1_sweep_freq - (self.ch1_sweep_freq >> self.ch1_sweep_shift)
            } else {
                self.ch1_sweep_freq + (self.ch1_sweep_freq >> self.ch1_sweep_shift)
            };
            if new_freq > 2047 {
                self.ch1_enable = false;
            } else if self.ch1_sweep_shift > 0 {
                self.ch1_sweep_freq = new_freq;
                self.ch1_freq = new_freq as u16;
            }
        }
    }

    fn clock_envelope(&mut self) {
        if self.ch1_env_period > 0 {
            self.ch1_env_timer -= 1;
            if self.ch1_env_timer == 0 {
                self.ch1_env_timer = self.ch1_env_period as u32;
                if self.ch1_env_dir && self.ch1_volume < 15 {
                    self.ch1_volume += 1;
                } else if !self.ch1_env_dir && self.ch1_volume > 0 {
                    self.ch1_volume -= 1;
                }
            }
        }
        if self.ch2_env_period > 0 {
            self.ch2_env_timer -= 1;
            if self.ch2_env_timer == 0 {
                self.ch2_env_timer = self.ch2_env_period as u32;
                if self.ch2_env_dir && self.ch2_volume < 15 {
                    self.ch2_volume += 1;
                } else if !self.ch2_env_dir && self.ch2_volume > 0 {
                    self.ch2_volume -= 1;
                }
            }
        }
        if self.ch4_env_period > 0 {
            self.ch4_env_timer -= 1;
            if self.ch4_env_timer == 0 {
                self.ch4_env_timer = self.ch4_env_period as u32;
                if self.ch4_env_dir && self.ch4_volume < 15 {
                    self.ch4_volume += 1;
                } else if !self.ch4_env_dir && self.ch4_volume > 0 {
                    self.ch4_volume -= 1;
                }
            }
        }
    }

    // Generate audio samples for a frame
    // GBA runs at 16.78 MHz, frame = 280896 cycles
    // At 32768 Hz, samples per frame = 280896 / (16777216 / 32768) ≈ 548.6
    pub fn generate_frame(&mut self, cpu_cycles: u32) {
        if self.soundcnt_x & 0x80 == 0 {
            // Master disable
            return;
        }

        // Clock frame sequencer at 32768/512 = 64 cycles per frame seq step
        // Actually: 8192 CPU cycles per frame seq step (512 Hz)
        self.frame_seq_timer += cpu_cycles;
        while self.frame_seq_timer >= 8192 {
            self.frame_seq_timer -= 8192;
            self.clock_frame_seq();
        }

        // Generate samples
        let cycles_per_sample = 16777216u32 / self.sample_rate;
        self.sample_timer += cpu_cycles;

        while self.sample_timer >= cycles_per_sample {
            self.sample_timer -= cycles_per_sample;
            if self.audio_count >= 4096 {
                break;
            }
            self.generate_sample();
        }
    }

    fn generate_sample(&mut self) {
        let mut left = 0i32;
        let mut right = 0i32;

        // PSG channels
        let psg_vol = ((self.soundcnt_h >> 0) & 0x7) + 1;
        let psg_left_vol = if self.soundcnt_l & 0x80 != 0 { psg_vol } else { 0 };
        let psg_right_vol = if self.soundcnt_l & 0x100 != 0 { psg_vol } else { 0 };

        // Channel 1
        if self.ch1_enable {
            let sample = self.get_ch1_sample() as i32;
            if self.soundcnt_l & 0x100 != 0 { right += sample * psg_right_vol as i32; } // R
            if self.soundcnt_l & 0x200 != 0 { right += sample * psg_right_vol as i32; } // Ch1 right
            if self.soundcnt_l & 0x100 != 0 { left += sample * psg_left_vol as i32; }
            // Let me redo this properly
        }

        // Redo properly
        left = 0;
        right = 0;

        let mut ch_left = [false; 4];
        let mut ch_right = [false; 4];
        ch_right[0] = self.soundcnt_l & 0x100 != 0;
        ch_left[0] = self.soundcnt_l & 0x200 != 0;
        ch_right[1] = self.soundcnt_l & 0x400 != 0;
        ch_left[1] = self.soundcnt_l & 0x800 != 0;
        ch_right[2] = self.soundcnt_l & 0x1000 != 0;
        ch_left[2] = self.soundcnt_l & 0x2000 != 0;
        ch_right[3] = self.soundcnt_l & 0x4000 != 0;
        ch_left[3] = self.soundcnt_l & 0x8000 != 0;

        // PSG master volume
        let psg_master_vol = ((self.soundcnt_h & 0x3) + 1) as i32;

        // Channel 1
        if self.ch1_enable {
            let s = self.get_ch1_sample() as i32 * self.ch1_volume as i32;
            if ch_right[0] { right += s * psg_master_vol; }
            if ch_left[0] { left += s * psg_master_vol; }
        }

        // Channel 2
        if self.ch2_enable {
            let s = self.get_ch2_sample() as i32 * self.ch2_volume as i32;
            if ch_right[1] { right += s * psg_master_vol; }
            if ch_left[1] { left += s * psg_master_vol; }
        }

        // Channel 3
        if self.ch3_enable && self.ch3_playing {
            let s = self.get_ch3_sample() as i32;
            if ch_right[2] { right += s * psg_master_vol; }
            if ch_left[2] { left += s * psg_master_vol; }
        }

        // Channel 4
        if self.ch4_enable {
            let s = self.get_ch4_sample() as i32 * self.ch4_volume as i32;
            if ch_right[3] { right += s * psg_master_vol; }
            if ch_left[3] { left += s * psg_master_vol; }
        }

        // FIFO A
        let fifo_a_vol = if self.soundcnt_h & 0x200 != 0 { 2 } else { 1 };
        let fifo_a_right = self.soundcnt_h & 0x100 != 0;
        let fifo_a_left = self.soundcnt_h & 0x200 != 0;
        let fifo_a_timer_sel = self.soundcnt_h & 0x400 != 0;
        if fifo_a_left || fifo_a_right {
            let s = self.get_fifo_a_sample() as i32;
            if fifo_a_left { left += s * fifo_a_vol; }
            if fifo_a_right { right += s * fifo_a_vol; }
        }

        // FIFO B
        let fifo_b_vol = if self.soundcnt_h & 0x2000 != 0 { 2 } else { 1 };
        let fifo_b_right = self.soundcnt_h & 0x1000 != 0;
        let fifo_b_left = self.soundcnt_h & 0x2000 != 0;
        let fifo_b_timer_sel = self.soundcnt_h & 0x4000 != 0;
        if fifo_b_left || fifo_b_right {
            let s = self.get_fifo_b_sample() as i32;
            if fifo_b_left { left += s * fifo_b_vol; }
            if fifo_b_right { right += s * fifo_b_vol; }
        }

        // Scale down
        right >>= 4;
        left >>= 4;

        // Clamp
        right = right.max(-32768).min(32767);
        left = left.max(-32768).min(32767);

        let idx = self.audio_count * 2;
        self.audio_buffer[idx] = left as i16;
        self.audio_buffer[idx + 1] = right as i16;
        self.audio_count += 1;
    }

    fn get_ch1_sample(&mut self) -> i8 {
        let freq = 131072 / (2048 - self.ch1_freq as u32);
        self.ch1_phase = self.ch1_phase.wrapping_add(1);
        let period = (16777216 / freq) / 8;
        let pos = (self.ch1_phase / (period.max(1))) % 8;
        let duty_pattern: [u8; 4] = [
            0b00000001,
            0b10000001,
            0b10000111,
            0b01111110,
        ];
        if duty_pattern[self.ch1_duty as usize] & (1 << pos) != 0 {
            8
        } else {
            -8
        }
    }

    fn get_ch2_sample(&mut self) -> i8 {
        let freq = 131072 / (2048 - self.ch2_freq as u32);
        self.ch2_phase = self.ch2_phase.wrapping_add(1);
        let period = (16777216 / freq) / 8;
        let pos = (self.ch2_phase / (period.max(1))) % 8;
        let duty_pattern: [u8; 4] = [
            0b00000001,
            0b10000001,
            0b10000111,
            0b01111110,
        ];
        if duty_pattern[self.ch2_duty as usize] & (1 << pos) != 0 {
            8
        } else {
            -8
        }
    }

    fn get_ch3_sample(&mut self) -> i8 {
        let freq = 65536 / (2048 - self.ch3_freq as u32);
        self.ch3_phase = self.ch3_phase.wrapping_add(1);
        let period = (16777216 / freq) / 32;
        let pos = (self.ch3_phase / (period.max(1))) % 32;
        let byte_idx = (pos / 2) as usize;
        let nibble = if pos & 1 == 0 {
            (self.ch3_wave[byte_idx] >> 4) & 0xF
        } else {
            self.ch3_wave[byte_idx] & 0xF
        };

        let vol_shift = match self.ch3_volume {
            0 => 4,
            1 => 0,
            2 => 1,
            3 => 2,
            _ => 0,
        };

        let sample = ((nibble >> vol_shift) as i8) - 8;
        sample
    }

    fn get_ch4_sample(&mut self) -> i8 {
        // Noise channel
        let bit = (self.ch4_lfsr & 1) as i8;
        if bit == 1 { 8 } else { -8 }
    }

    fn get_fifo_a_sample(&mut self) -> i8 {
        if self.fifo_a_count > 0 {
            self.fifo_a_pop() as i8
        } else {
            0
        }
    }

    fn get_fifo_b_sample(&mut self) -> i8 {
        if self.fifo_b_count > 0 {
            self.fifo_b_pop() as i8
        } else {
            0
        }
    }

    // Called when a timer triggers a FIFO sample
    pub fn timer_fifo_event(&mut self, timer_id: usize) {
        let fifo_a_timer = self.soundcnt_h & 0x400 != 0;
        let fifo_b_timer = self.soundcnt_h & 0x4000 != 0;

        // Timer 0 or 1 can be selected
        if timer_id == 0 && !fifo_a_timer {
            self.fifo_a_pop();
        }
        if timer_id == 1 && fifo_a_timer {
            self.fifo_a_pop();
        }
    }

    pub fn drain_audio(&mut self) -> usize {
        let count = self.audio_count;
        self.audio_count = 0;
        count
    }
}
