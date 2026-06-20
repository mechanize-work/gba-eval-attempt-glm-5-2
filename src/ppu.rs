// PPU - Pixel Processing Unit
// Renders GBA graphics to a framebuffer
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

use crate::memory::*;

pub const SCREEN_W: usize = 240;
pub const SCREEN_H: usize = 160;
pub const FB_SIZE: usize = SCREEN_W * SCREEN_H;

// IO register offsets from IO_BASE
const DISPCNT: usize = 0x00;
const DISPSTAT: usize = 0x04;
const VCOUNT: usize = 0x06;
const BG0CNT: usize = 0x08;
const BG1CNT: usize = 0x0A;
const BG2CNT: usize = 0x0C;
const BG3CNT: usize = 0x0E;
const BG0HOFS: usize = 0x10;
const BG0VOFS: usize = 0x12;
const BG1HOFS: usize = 0x14;
const BG1VOFS: usize = 0x16;
const BG2HOFS: usize = 0x18;
const BG2VOFS: usize = 0x1A;
const BG3HOFS: usize = 0x1C;
const BG3VOFS: usize = 0x1E;
const BG2PA: usize = 0x20;
const BG2PB: usize = 0x22;
const BG2PC: usize = 0x24;
const BG2PD: usize = 0x26;
const BG2X_L: usize = 0x28;
const BG2X_H: usize = 0x2A;
const BG2Y_L: usize = 0x2C;
const BG2Y_H: usize = 0x2E;
const BG3PA: usize = 0x30;
const BG3PB: usize = 0x32;
const BG3PC: usize = 0x34;
const BG3PD: usize = 0x36;
const BG3X_L: usize = 0x38;
const BG3X_H: usize = 0x3A;
const BG3Y_L: usize = 0x3C;
const BG3Y_H: usize = 0x3E;
const WIN0H: usize = 0x40;
const WIN1H: usize = 0x42;
const WIN0V: usize = 0x44;
const WIN1V: usize = 0x46;
const WININ: usize = 0x48;
const WINOUT: usize = 0x4A;
const MOSAIC: usize = 0x4C;
const BLDCNT: usize = 0x50;
const BLDALPHA: usize = 0x52;
const BLDY: usize = 0x54;

pub struct Ppu {
    pub framebuffer: Box<[u32; FB_SIZE]>,
    pub current_line: u16,
    pub cycle: u32,
    pub frame_complete: bool,
    pub bg2x: i32,
    pub bg2y: i32,
    pub bg3x: i32,
    pub bg3y: i32,
    // Snapshot of display registers at VBlank start for rendering
    pub snap_dispcnt: u16,
    pub snap_bgcnt: [u16; 4],
    pub snap_bg_hofs: [u16; 4],
    pub snap_bg_vofs: [u16; 4],
    pub snap_bldcnt: u16,
    pub snap_bldalpha: u16,
    pub snap_bldy: u16,
    pub snap_mosaic: u16,
    pub snap_win0h: u16,
    pub snap_win1h: u16,
    pub snap_win0v: u16,
    pub snap_win1v: u16,
    pub snap_winin: u16,
    pub snap_winout: u16,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            framebuffer: Box::new([0; FB_SIZE]),
            current_line: 0,
            cycle: 0,
            frame_complete: false,
            bg2x: 0,
            bg2y: 0,
            bg3x: 0,
            bg3y: 0,
            snap_dispcnt: 0,
            snap_bgcnt: [0; 4],
            snap_bg_hofs: [0; 4],
            snap_bg_vofs: [0; 4],
            snap_bldcnt: 0,
            snap_bldalpha: 0,
            snap_bldy: 0,
            snap_mosaic: 0,
            snap_win0h: 0,
            snap_win1h: 0,
            snap_win0v: 0,
            snap_win1v: 0,
            snap_winin: 0,
            snap_winout: 0,
        }
    }

    pub fn reset(&mut self) {
        for p in self.framebuffer.iter_mut() { *p = 0; }
        self.current_line = 0;
        self.cycle = 0;
        self.frame_complete = false;
        self.bg2x = 0;
        self.bg2y = 0;
        self.bg3x = 0;
        self.bg3y = 0;
        self.snap_dispcnt = 0;
        self.snap_bgcnt = [0; 4];
        self.snap_bg_hofs = [0; 4];
        self.snap_bg_vofs = [0; 4];
        self.snap_bldcnt = 0;
        self.snap_bldalpha = 0;
        self.snap_bldy = 0;
        self.snap_mosaic = 0;
        self.snap_win0h = 0;
        self.snap_win1h = 0;
        self.snap_win0v = 0;
        self.snap_win1v = 0;
        self.snap_winin = 0;
        self.snap_winout = 0;
    }

    #[inline]
    fn io_read_half(&self, mem: &Memory, off: usize) -> u16 {
        (mem.io[off] as u16) | ((mem.io[off + 1] as u16) << 8)
    }

    // Render a complete frame
    pub fn render_frame(&mut self, mem: &Memory) {
        // Use snapshotted display registers (captured at VBlank start)
        let dispcnt = self.snap_dispcnt;
        let mode = dispcnt & 0x7;
        let bg_en = (dispcnt >> 8) & 0xF;
        let obj_en = (dispcnt >> 12) & 1;

        let forcelen = dispcnt & 0x4000 != 0;
        if forcelen {
            for i in 0..FB_SIZE {
                self.framebuffer[i] = 0xFF000000;
            }
            return;
        }

        let bgcnt = self.snap_bgcnt;
        let win0h = self.snap_win0h;
        let win1h = self.snap_win1h;
        let win0v = self.snap_win0v;
        let win1v = self.snap_win1v;
        let winin = self.snap_winin;
        let winout = self.snap_winout;
        let bldcnt = self.snap_bldcnt;
        let bldalpha = self.snap_bldalpha;
        let bldy = self.snap_bldy & 0x1F;

        // Read mosaic
        let mosaic = self.snap_mosaic;

        // Read scroll offsets
        let bg_hofs = [
            self.snap_bg_hofs[0] & 0x1FF,
            self.snap_bg_hofs[1] & 0x1FF,
            self.snap_bg_hofs[2] & 0x1FF,
            self.snap_bg_hofs[3] & 0x1FF,
        ];
        let bg_vofs = [
            self.snap_bg_vofs[0] & 0x1FF,
            self.snap_bg_vofs[1] & 0x1FF,
            self.snap_bg_vofs[2] & 0x1FF,
            self.snap_bg_vofs[3] & 0x1FF,
        ];

        // Render line by line
        for line in 0..SCREEN_H {
            self.render_line(mem, line as u16, mode, bg_en, obj_en, &bgcnt, &bg_hofs, &bg_vofs, &win0h, &win1h, &win0v, &win1v, winin, winout, bldcnt, bldalpha, bldy, mosaic, dispcnt);
        }
    }

    fn render_line(
        &mut self,
        mem: &Memory,
        line: u16,
        mode: u16,
        bg_en: u16,
        obj_en: u16,
        bgcnt: &[u16; 4],
        bg_hofs: &[u16; 4],
        bg_vofs: &[u16; 4],
        win0h: &u16,
        win1h: &u16,
        win0v: &u16,
        win1v: &u16,
        winin: u16,
        winout: u16,
        bldcnt: u16,
        bldalpha: u16,
        bldy: u16,
        mosaic: u16,
        dispcnt: u16,
    ) {
        // Per-line buffers for each layer
        // Each pixel: (color_idx, is_opaque, priority)
        let mut bg_pixels: [[Pixel; SCREEN_W]; 4] = [[Pixel::default(); SCREEN_W]; 4];
        let mut obj_pixels: [Pixel; SCREEN_W] = [Pixel::default(); SCREEN_W];

        // Render each enabled background
        for bg in 0..4 {
            if bg_en & (1 << bg) == 0 {
                continue;
            }
            match mode {
                0 => {
                    self.render_text_bg(mem, bg, line, bgcnt[bg], bg_hofs[bg], bg_vofs[bg], &mut bg_pixels[bg]);
                }
                1 => {
                    match bg {
                        0 | 1 => self.render_text_bg(mem, bg, line, bgcnt[bg], bg_hofs[bg], bg_vofs[bg], &mut bg_pixels[bg]),
                        2 => self.render_affine_bg(mem, bg, line, bgcnt[bg], &mut bg_pixels[bg]),
                        _ => {}
                    }
                }
                2 => {
                    match bg {
                        2 | 3 => self.render_affine_bg(mem, bg, line, bgcnt[bg], &mut bg_pixels[bg]),
                        _ => {}
                    }
                }
                3 => {
                    if bg == 2 {
                        self.render_bitmap_bg(mem, bg, line, bgcnt[bg], &mut bg_pixels[bg], 3);
                    }
                }
                4 => {
                    if bg == 2 {
                        self.render_bitmap_bg(mem, bg, line, bgcnt[bg], &mut bg_pixels[bg], 4);
                    }
                }
                5 => {
                    if bg == 2 {
                        self.render_bitmap_bg(mem, bg, line, bgcnt[bg], &mut bg_pixels[bg], 5);
                    }
                }
                _ => {}
            }
        }

        // Render sprites
        if obj_en != 0 {
            self.render_sprites(mem, line, &mut obj_pixels, dispcnt);
        }

        // Composite all layers
        self.composite_line(line, &bg_pixels, &obj_pixels, bg_en, obj_en, bldcnt, bldalpha, bldy, mem, win0h, win1h, win0v, win1v, winin, winout, dispcnt);
    }

    fn render_text_bg(
        &self,
        mem: &Memory,
        bg: usize,
        line: u16,
        bgcnt: u16,
        hofs: u16,
        vofs: u16,
        pixels: &mut [Pixel; SCREEN_W],
    ) {
        let priority = bgcnt & 3;
        let char_base = ((bgcnt >> 2) & 0x3) as u32 * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
        let mosaic = bgcnt & 0x40 != 0;
        let palette_256 = bgcnt & 0x80 != 0;
        let wrap = bgcnt & 0x2000 != 0;

        let screen_entry_addr = 0x0600_0000 + screen_base;
        let char_addr = 0x0600_0000 + char_base;

        let y = line as u32 + vofs as u32;
        let tile_y = (y / 8) & 31;
        let pixel_y = (y & 7) as u32;

        for x in 0..SCREEN_W {
            let screen_x = (x as u32 + hofs as u32) & 511;
            let tile_x = (screen_x / 8) & 31;
            let pixel_x = screen_x & 7;

            // Map size
            let map_size = (bgcnt >> 14) & 0x3;
            let (map_w, map_h) = match map_size {
                0 => (32, 32),
                1 => (64, 32),
                2 => (32, 64),
                3 => (64, 64),
                _ => (32, 32),
            };

            // Determine which 32x32 block we're in
            let block_x = if screen_x >= 256 { 1 } else { 0 };
            let block_y = if y >= 256 { 1 } else { 0 };

            let block_offset = match map_size {
                0 => 0,
                1 => block_x * 0x800,
                2 => block_y * 0x800,
                3 => (block_y * 2 + block_x) * 0x800,
                _ => 0,
            };

            // Read screen entry (tile number + attributes)
            let entry_addr = screen_entry_addr + block_offset + ((tile_y * 32 + tile_x) * 2) as u32;
            let entry = mem.read_vram_half(entry_addr & 0x06FF_FFFF);

            let tile_num = entry & 0x3FF;
            let hflip = entry & 0x400 != 0;
            let vflip = entry & 0x800 != 0;
            let palette_bank = ((entry >> 12) & 0xF) as u32;

            // Read pixel data
            let px = if hflip { 7 - pixel_x } else { pixel_x };
            let py = if vflip { 7 - pixel_y } else { pixel_y };

            if palette_256 {
                // 256-color tile (1 byte per pixel)
                let tile_offset = (tile_num as u32) * 64 + py * 8 + px;
                let addr = char_addr + tile_offset;
                let color_idx = mem.read_vram_byte(addr & 0x06FF_FFFF) as u32;
                if color_idx != 0 {
                    let palette_addr = 0x0500_0000 + color_idx * 2;
                    let color = mem.read_palette_half(palette_addr);
                    pixels[x] = Pixel {
                        color: rgb5_to_rgb555(color),
                        opaque: true,
                        priority,
                    };
                }
            } else {
                // 16-color tile (4bpp)
                let tile_offset = (tile_num as u32) * 32 + py * 4 + (px / 2);
                let addr = char_addr + tile_offset;
                let byte = mem.read_vram_byte(addr & 0x06FF_FFFF);
                let color_idx = if px & 1 == 0 {
                    byte & 0xF
                } else {
                    (byte >> 4) & 0xF
                };
                if color_idx != 0 {
                    let palette_addr = 0x0500_0000 + (palette_bank * 16 + color_idx as u32) * 2;
                    let color = mem.read_palette_half(palette_addr);
                    pixels[x] = Pixel {
                        color: rgb5_to_rgb555(color),
                        opaque: true,
                        priority,
                    };
                }
            }
        }
    }

    fn render_affine_bg(
        &self,
        mem: &Memory,
        bg: usize,
        line: u16,
        bgcnt: u16,
        pixels: &mut [Pixel; SCREEN_W],
    ) {
        let priority = bgcnt & 3;
        let char_base = ((bgcnt >> 2) & 0x3) as u32 * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x100;
        let wrap = bgcnt & 0x2000 != 0;
        let map_size = (bgcnt >> 14) & 0x3;
        let map_dim: u32 = 16 << map_size; // 16, 32, 64, 128

        // Read affine parameters
        let (pa, pb, pc, pd, ref_x, ref_y) = if bg == 2 {
            (
                self.io_read_half(mem, BG2PA) as i16 as i32,
                self.io_read_half(mem, BG2PB) as i16 as i32,
                self.io_read_half(mem, BG2PC) as i16 as i32,
                self.io_read_half(mem, BG2PD) as i16 as i32,
                ((self.io_read_half(mem, BG2X_L) as u32) | ((self.io_read_half(mem, BG2X_H) as u32 & 0x0FFF) << 16)) as i32,
                ((self.io_read_half(mem, BG2Y_L) as u32) | ((self.io_read_half(mem, BG2Y_H) as u32 & 0x0FFF) << 16)) as i32,
            )
        } else {
            (
                self.io_read_half(mem, BG3PA) as i16 as i32,
                self.io_read_half(mem, BG3PB) as i16 as i32,
                self.io_read_half(mem, BG3PC) as i16 as i32,
                self.io_read_half(mem, BG3PD) as i16 as i32,
                ((self.io_read_half(mem, BG3X_L) as u32) | ((self.io_read_half(mem, BG3X_H) as u32 & 0x0FFF) << 16)) as i32,
                ((self.io_read_half(mem, BG3Y_L) as u32) | ((self.io_read_half(mem, BG3Y_H) as u32 & 0x0FFF) << 16)) as i32,
            )
        };

        // Sign extend 28-bit to 32-bit
        let ref_x = (ref_x << 4) >> 4;
        let ref_y = (ref_y << 4) >> 4;

        // Internal reference point for this line
        let mut x = ref_x;
        let mut y = ref_y;

        let screen_base_addr = 0x0600_0000 + screen_base;
        let char_addr = 0x0600_0000 + char_base;

        for px in 0..SCREEN_W {
            // Get pixel coordinates in the background map
            let mx = (x >> 8) as i32;
            let my = (y >> 8) as i32;

            let (mx, my) = if wrap {
                (mx & ((map_dim * 8 - 1) as i32), my & ((map_dim * 8 - 1) as i32))
            } else {
                if mx < 0 || mx >= (map_dim * 8) as i32 || my < 0 || my >= (map_dim * 8) as i32 {
                    x += pa;
                    y += pc;
                    continue;
                }
                (mx, my)
            };

            let tile_x = ((mx >> 3) as u32) & (map_dim - 1);
            let tile_y = ((my >> 3) as u32) & (map_dim - 1);
            let pixel_x = (mx & 7) as u32;
            let pixel_y = (my & 7) as u32;

            // Read screen entry (8-bit for affine)
            let entry_addr = screen_base_addr + tile_y * map_dim + tile_x;
            let tile_num = mem.read_vram_byte(entry_addr & 0x06FF_FFFF) as u32;

            // Read pixel data (8bpp, 256-color)
            let tile_offset = tile_num * 64 + pixel_y * 8 + pixel_x;
            let addr = char_addr + tile_offset;
            let color_idx = mem.read_vram_byte(addr & 0x06FF_FFFF) as u32;

            if color_idx != 0 {
                let palette_addr = 0x0500_0000 + color_idx * 2;
                let color = mem.read_palette_half(palette_addr);
                pixels[px] = Pixel {
                    color: rgb5_to_rgb555(color),
                    opaque: true,
                    priority,
                };
            }

            x += pa;
            y += pc;
        }
    }

    fn render_bitmap_bg(
        &self,
        mem: &Memory,
        bg: usize,
        line: u16,
        bgcnt: u16,
        pixels: &mut [Pixel; SCREEN_W],
        mode: u16,
    ) {
        let priority = bgcnt & 3;
        let page = (bgcnt >> 4) & 0x1; // for modes 4,5

        match mode {
            3 => {
                // Mode 3: 240x160, 16-bit color (RGB555)
                let base = 0x0600_0000;
                let line_offset = (line as u32) * 240 * 2;
                for x in 0..SCREEN_W {
                    let addr = base + line_offset + (x as u32) * 2;
                    let color = mem.read_vram_half(addr & 0x06FF_FFFF);
                    pixels[x] = Pixel {
                        color: rgb5_to_rgb555(color),
                        opaque: color != 0 || true, // In bitmap mode, all pixels are opaque
                        priority,
                    };
                }
            }
            4 => {
                // Mode 4: 240x160, 8-bit indexed, 2 pages
                let base = 0x0600_0000 + (page as u32) * 0xA000;
                let line_offset = (line as u32) * 240;
                for x in 0..SCREEN_W {
                    let addr = base + line_offset + x as u32;
                    let color_idx = mem.read_vram_byte(addr & 0x06FF_FFFF) as u32;
                    if color_idx != 0 {
                        let palette_addr = 0x0500_0000 + color_idx * 2;
                        let color = mem.read_palette_half(palette_addr);
                        pixels[x] = Pixel {
                            color: rgb5_to_rgb555(color),
                            opaque: true,
                            priority,
                        };
                    }
                }
            }
            5 => {
                // Mode 5: 160x128, 16-bit color, 2 pages
                let base = 0x0600_0000 + (page as u32) * 0xA000;
                let line_offset = (line as u32) * 160 * 2;
                for x in 0..SCREEN_W {
                    if x >= 160 || line >= 128 {
                        continue;
                    }
                    let addr = base + line_offset + (x as u32) * 2;
                    let color = mem.read_vram_half(addr & 0x06FF_FFFF);
                    pixels[x] = Pixel {
                        color: rgb5_to_rgb555(color),
                        opaque: true,
                        priority,
                    };
                }
            }
            _ => {}
        }
    }

    fn render_sprites(
        &self,
        mem: &Memory,
        line: u16,
        pixels: &mut [Pixel; SCREEN_W],
        dispcnt: u16,
    ) {
        let obj_mode = (dispcnt >> 6) & 0x3; // 0=1D, 1=2D
        let hblank_free = dispcnt & 0x20 != 0;

        // OAM is at 0x0700_0000, 1KB, 128 sprites
        for i in 0..128 {
            let oam_addr = (i * 8) as u32;

            let attr0 = mem.read_oam_half(0x0700_0000 + oam_addr);
            let attr1 = mem.read_oam_half(0x0700_0000 + oam_addr + 2);
            let attr2 = mem.read_oam_half(0x0700_0000 + oam_addr + 4);

            let y = (attr0 & 0xFF) as i32;
            let obj_shape = (attr0 >> 14) & 0x3;

            let x = (attr1 & 0x1FF) as i32;
            let obj_size = (attr1 >> 14) & 0x3;

            // Size lookup
            let (w, h) = match (obj_shape, obj_size) {
                (0, 0) => (8, 8),
                (0, 1) => (16, 16),
                (0, 2) => (32, 32),
                (0, 3) => (64, 64),
                (1, 0) => (16, 8),
                (1, 1) => (32, 8),
                (1, 2) => (32, 16),
                (1, 3) => (64, 32),
                (2, 0) => (8, 16),
                (2, 1) => (8, 32),
                (2, 2) => (16, 32),
                (2, 3) => (32, 64),
                _ => (8, 8),
            };

            // Y position with wrapping
            let y_pos = if y >= 192 { y - 256 } else { y };
            let dy = line as i32 - y_pos;
            if dy < 0 || dy >= h {
                continue;
            }

            // Check if sprite is disabled
            let attr0_mode = (attr0 >> 10) & 0x3;
            if attr0_mode == 2 {
                continue; // Disabled
            }

            let mosaic = attr0 & 0x1000 != 0;
            let is_8bpp = attr0 & 0x2000 != 0;
            let double_size = attr0 & 0x4000 != 0;
            let obj_disabled = attr0 & 0x8000 != 0; // Actually this is the "disabled" bit for rotated
            if obj_disabled && !double_size {
                // For non-affine: bit 15 is unused
            }

            let hflip = attr1 & 0x1000 != 0;
            let vflip = attr1 & 0x2000 != 0;
            let palette_bank = ((attr2 >> 12) & 0xF) as u32;
            let priority = (attr2 >> 10) & 0x3;
            let tile_num = (attr2 & 0x3FF) as u32;

            // VFlip
            let dy = if vflip { h - 1 - dy } else { dy };
            let dy = dy as u32;

            let is_affine = attr0_mode == 1;
            let affine_idx = if is_affine {
                ((attr1 >> 9) & 0x1F) as usize
            } else {
                0
            };

            // Tile data address
            let char_base = ((dispcnt >> 4) & 0x3) as u32 * 0x4000;
            let char_addr = 0x0601_0000 + char_base;

            let w_u32 = w as u32;

            // For each pixel in the sprite on this line
            for dx in 0..w {
                let screen_x = x + dx;
                let actual_x = if screen_x >= 240 { screen_x - 512 } else { screen_x };
                if actual_x < 0 || actual_x >= 240 {
                    continue;
                }

                let dx2 = if hflip { w - 1 - dx } else { dx };
                let dx2 = dx2 as u32;

                if is_8bpp {
                    // 8bpp: 64 bytes per tile
                    let tile_idx = tile_num + (dy / 8) * (w_u32 / 8) + (dx2 / 8);
                    let tile_offset = tile_idx * 64 + (dy % 8) * 8 + (dx2 % 8);

                    if obj_mode == 0 {
                        // 1D mapping
                        let addr = char_addr + tile_offset;
                        let color_idx = mem.read_vram_byte(addr & 0x06FF_FFFF) as u32;
                        if color_idx != 0 {
                            let palette_addr = 0x0500_0200 + color_idx * 2;
                            let color = mem.read_palette_half(palette_addr);
                            pixels[actual_x as usize] = Pixel {
                                color: rgb5_to_rgb555(color),
                                opaque: true,
                                priority,
                            };
                        }
                    } else {
                        // 2D mapping
                        let tile_row = tile_num / 32 + dy / 8;
                        let tile_col = (tile_num % 32) + dx2 / 8;
                        let tile_idx = tile_row * 32 + tile_col;
                        let tile_offset = tile_idx * 64 + (dy % 8) * 8 + (dx2 % 8);
                        let addr = char_addr + tile_offset;
                        let color_idx = mem.read_vram_byte(addr & 0x06FF_FFFF) as u32;
                        if color_idx != 0 {
                            let palette_addr = 0x0500_0200 + color_idx * 2;
                            let color = mem.read_palette_half(palette_addr);
                            pixels[actual_x as usize] = Pixel {
                                color: rgb5_to_rgb555(color),
                                opaque: true,
                                priority,
                            };
                        }
                    }
                } else {
                    // 4bpp: 32 bytes per tile
                    let tile_idx = tile_num + (dy / 8) * (w_u32 / 8) + (dx2 / 8);
                    let tile_offset = tile_idx * 32 + (dy % 8) * 4 + (dx2 / 2);
                    let addr = char_addr + tile_offset;
                    let byte = mem.read_vram_byte(addr & 0x06FF_FFFF);
                    let color_idx = if dx2 & 1 == 0 {
                        byte & 0xF
                    } else {
                        (byte >> 4) & 0xF
                    };
                    if color_idx != 0 {
                        let palette_addr = 0x0500_0200 + (palette_bank * 16 + color_idx as u32) * 2;
                        let color = mem.read_palette_half(palette_addr);
                        pixels[actual_x as usize] = Pixel {
                            color: rgb5_to_rgb555(color),
                            opaque: true,
                            priority,
                        };
                    }
                }
            }
        }
    }

    fn composite_line(
        &mut self,
        line: u16,
        bg_pixels: &[[Pixel; SCREEN_W]; 4],
        obj_pixels: &[Pixel; SCREEN_W],
        bg_en: u16,
        obj_en: u16,
        bldcnt: u16,
        bldalpha: u16,
        bldy: u16,
        mem: &Memory,
        win0h: &u16,
        win1h: &u16,
        win0v: &u16,
        win1v: &u16,
        winin: u16,
        winout: u16,
        dispcnt: u16,
    ) {
        let backdrop_color = mem.read_palette_half(0x0500_0000);
        let backdrop = rgb5_to_rgb555(backdrop_color);

        let win0_en = dispcnt & 0x2000 != 0;
        let win1_en = dispcnt & 0x4000 != 0;
        let winobj_en = dispcnt & 0x8000 != 0;

        // Window ranges
        let win0_x1 = (win0h & 0xFF) as usize;
        let win0_x2 = ((win0h >> 8) & 0xFF) as usize;
        let win0_y1 = (win0v & 0xFF) as usize;
        let win0_y2 = ((win0v >> 8) & 0xFF) as usize;

        let win1_x1 = (win1h & 0xFF) as usize;
        let win1_x2 = ((win1h >> 8) & 0xFF) as usize;
        let win1_y1 = (win1v & 0xFF) as usize;
        let win1_y2 = ((win1v >> 8) & 0xFF) as usize;

        let line_idx = line as usize;
        let fb_offset = line_idx * SCREEN_W;

        for x in 0..SCREEN_W {
            // Determine window for this pixel
            let mut win_layer_en: u16;
            let mut win_blend_en = false;

            if win0_en {
                let y_in = if win0_y1 <= win0_y2 {
                    (line as usize) >= win0_y1 && (line as usize) < win0_y2
                } else {
                    (line as usize) >= win0_y1 || (line as usize) < win0_y2
                };
                let x_in = if win0_x1 <= win0_x2 {
                    x >= win0_x1 && x < win0_x2
                } else {
                    x >= win0_x1 || x < win0_x2
                };
                if y_in && x_in {
                    win_layer_en = winin & 0x3F;
                    win_blend_en = winin & 0x40 != 0;
                    return self.composite_pixel(x, fb_offset, bg_pixels, obj_pixels, bg_en, obj_en, bldcnt, bldalpha, bldy, backdrop, win_layer_en, win_blend_en);
                }
            }

            if win1_en {
                let y_in = if win1_y1 <= win1_y2 {
                    (line as usize) >= win1_y1 && (line as usize) < win1_y2
                } else {
                    (line as usize) >= win1_y1 || (line as usize) < win1_y2
                };
                let x_in = if win1_x1 <= win1_x2 {
                    x >= win1_x1 && x < win1_x2
                } else {
                    x >= win1_x1 || x < win1_x2
                };
                if y_in && x_in {
                    win_layer_en = (winin >> 8) & 0x3F;
                    win_blend_en = winin & 0x4000 != 0;
                    return self.composite_pixel(x, fb_offset, bg_pixels, obj_pixels, bg_en, obj_en, bldcnt, bldalpha, bldy, backdrop, win_layer_en, win_blend_en);
                }
            }

            // WinObj
            if winobj_en && obj_pixels[x].opaque {
                win_layer_en = winout & 0x3F;
                win_blend_en = winout & 0x40 != 0;
                return self.composite_pixel(x, fb_offset, bg_pixels, obj_pixels, bg_en, obj_en, bldcnt, bldalpha, bldy, backdrop, win_layer_en, win_blend_en);
            }

            // Outside all windows
            win_layer_en = (winout >> 8) & 0x3F;
            win_blend_en = winout & 0x4000 != 0;
            self.composite_pixel(x, fb_offset, bg_pixels, obj_pixels, bg_en, obj_en, bldcnt, bldalpha, bldy, backdrop, win_layer_en, win_blend_en);
        }
    }

    fn composite_pixel(
        &mut self,
        x: usize,
        fb_offset: usize,
        bg_pixels: &[[Pixel; SCREEN_W]; 4],
        obj_pixels: &[Pixel; SCREEN_W],
        bg_en: u16,
        obj_en: u16,
        bldcnt: u16,
        bldalpha: u16,
        bldy: u16,
        backdrop: u16,
        win_layer_en: u16,
        win_blend_en: bool,
    ) {
        // Find topmost opaque pixel
        // Priority: 0 is highest, 3 is lowest
        // Layers with same priority: lower layer number wins

        struct LayerPixel {
            color: u16,
            is_obj: bool,
            bg_idx: u16,
        }

        let mut candidates: [Option<LayerPixel>; 5] = [None, None, None, None, None];

        // Backdrop is always at the bottom
        candidates[4] = Some(LayerPixel { color: backdrop, is_obj: false, bg_idx: 4 });

        // Check BG layers
        for bg in 0..4 {
            if bg_en & (1 << bg) != 0 && win_layer_en & (1 << bg) != 0 {
                if bg_pixels[bg][x].opaque {
                    candidates[bg] = Some(LayerPixel {
                        color: bg_pixels[bg][x].color,
                        is_obj: false,
                        bg_idx: bg as u16,
                    });
                }
            }
        }

        // Check OBJ layer
        if obj_en != 0 && win_layer_en & 0x10 != 0 && obj_pixels[x].opaque {
            candidates[4] = Some(LayerPixel {
                color: obj_pixels[x].color,
                is_obj: true,
                bg_idx: 4,
            });
        }

        // Find topmost by priority
        let mut top: Option<LayerPixel> = None;
        let mut top_priority = 4u16;

        for bg in 0..5 {
            if let Some(ref lp) = candidates[bg] {
                let prio = if lp.is_obj {
                    obj_pixels[x].priority
                } else if lp.bg_idx < 4 {
                    bg_pixels[lp.bg_idx as usize][x].priority
                } else {
                    4 // backdrop
                };

                if prio < top_priority || (prio == top_priority && top.is_none()) {
                    top_priority = prio;
                    top = Some(LayerPixel {
                        color: lp.color,
                        is_obj: lp.is_obj,
                        bg_idx: lp.bg_idx,
                    });
                }
            }
        }

        // Actually, let me redo this properly.
        // We need to find the topmost and second-topmost opaque pixels for blending.
        let mut top_color = backdrop;
        let mut top_is_obj = false;
        let mut top_bg = 4u16;
        let mut top_prio = 4u16;
        let mut second_color = backdrop;

        // Collect all visible pixels with priorities
        let mut layers: Vec<(u16, u16, u16, bool)> = Vec::new(); // (priority, color, bg_idx, is_obj)

        // Backdrop
        layers.push((4, backdrop, 4, false));

        // BGs
        for bg in 0..4 {
            if bg_en & (1 << bg) != 0 && win_layer_en & (1 << bg) != 0 && bg_pixels[bg][x].opaque {
                layers.push((bg_pixels[bg][x].priority, bg_pixels[bg][x].color, bg as u16, false));
            }
        }

        // OBJ
        if obj_en != 0 && win_layer_en & 0x10 != 0 && obj_pixels[x].opaque {
            layers.push((obj_pixels[x].priority, obj_pixels[x].color, 4, true));
        }

        // Sort by priority (ascending), then by layer order (BG0 before BG1 etc, backdrop last)
        // Within same priority, BG0 > BG1 > BG2 > BG3 > OBJ
        layers.sort_by(|a, b| {
            if a.0 != b.0 {
                a.0.cmp(&b.0)
            } else {
                // Same priority: lower bg_idx first, but OBJ is after BGs
                if a.3 && !b.3 {
                    core::cmp::Ordering::Greater
                } else if !a.3 && b.3 {
                    core::cmp::Ordering::Less
                } else {
                    a.2.cmp(&b.2)
                }
            }
        });

        top_color = layers[0].1;
        top_is_obj = layers[0].3;
        top_bg = layers[0].2;
        top_prio = layers[0].0;

        if layers.len() > 1 {
            second_color = layers[1].1;
        }

        // Blending
        let blend_mode = (bldcnt >> 6) & 0x3;
        let top_blend_target = if top_is_obj {
            bldcnt & 0x40 != 0
        } else {
            (bldcnt >> (top_bg as u16)) & 1 != 0
        };

        let mut final_color = top_color;

        if blend_mode != 0 && (win_blend_en || true) {
            // Check if this layer is a blend target
            let is_target1 = if top_is_obj {
                bldcnt & 0x40 != 0
            } else if top_bg < 4 {
                (bldcnt >> top_bg) & 1 != 0
            } else {
                bldcnt & 0x1 != 0 // backdrop
            };

            if is_target1 {
                match blend_mode {
                    1 => {
                        // Alpha blending
                        let is_target2 = bldcnt & 0x3E00 != 0; // any second target
                        // Check second target properly
                        let second_is_target = if layers.len() > 1 {
                            let (_, _, second_bg, second_obj) = layers[1];
                            if second_obj {
                                bldcnt & 0x4000 != 0
                            } else if second_bg < 4 {
                                (bldcnt >> (8 + second_bg)) & 1 != 0
                            } else {
                                bldcnt & 0x100 != 0 // backdrop as 2nd target
                            }
                        } else {
                            false
                        };

                        if second_is_target || true {
                            let eva = (bldalpha & 0x1F).min(16);
                            let evb = ((bldalpha >> 8) & 0x1F).min(16);
                            final_color = blend_alpha(top_color, second_color, eva, evb);
                        }
                    }
                    2 => {
                        // Brightness increase
                        let evy = bldy.min(16);
                        final_color = blend_bright(top_color, evy, true);
                    }
                    3 => {
                        // Brightness decrease
                        let evy = bldy.min(16);
                        final_color = blend_bright(top_color, evy, false);
                    }
                    _ => {}
                }
            }
        }

        // Convert to ABGR
        self.framebuffer[fb_offset + x] = rgb555_to_abgr(final_color);
    }
}

#[derive(Clone, Copy, Default)]
pub struct Pixel {
    pub color: u16,  // RGB555
    pub opaque: bool,
    pub priority: u16,
}

#[inline]
fn rgb5_to_rgb555(c: u16) -> u16 {
    c & 0x7FFF
}

#[inline]
fn rgb555_to_abgr(c: u16) -> u32 {
    let r = (c & 0x1F) as u32;
    let g = ((c >> 5) & 0x1F) as u32;
    let b = ((c >> 10) & 0x1F) as u32;
    // Convert 5-bit to 8-bit
    let r8 = (r << 3) | (r >> 2);
    let g8 = (g << 3) | (g >> 2);
    let b8 = (b << 3) | (b >> 2);
    0xFF00_0000 | (b8 << 16) | (g8 << 8) | r8
}

#[inline]
fn blend_alpha(top: u16, bottom: u16, eva: u16, evb: u16) -> u16 {
    let eva = eva as u32;
    let evb = evb as u32;
    let tr = (top & 0x1F) as u32;
    let tg = ((top >> 5) & 0x1F) as u32;
    let tb = ((top >> 10) & 0x1F) as u32;
    let br = (bottom & 0x1F) as u32;
    let bg = ((bottom >> 5) & 0x1F) as u32;
    let bb = ((bottom >> 10) & 0x1F) as u32;

    let r = (tr * eva + br * evb) / 16;
    let g = (tg * eva + bg * evb) / 16;
    let b = (tb * eva + bb * evb) / 16;

    (r.min(31) as u16) | ((g.min(31) as u16) << 5) | ((b.min(31) as u16) << 10)
}

#[inline]
fn blend_bright(c: u16, evy: u16, increase: bool) -> u16 {
    let evy = evy as u32;
    let r = (c & 0x1F) as u32;
    let g = ((c >> 5) & 0x1F) as u32;
    let b = ((c >> 10) & 0x1F) as u32;

    let (r2, g2, b2) = if increase {
        (
            r + ((31 - r) * evy) / 16,
            g + ((31 - g) * evy) / 16,
            b + ((31 - b) * evy) / 16,
        )
    } else {
        (
            r - (r * evy) / 16,
            g - (g * evy) / 16,
            b - (b * evy) / 16,
        )
    };

    (r2.min(31) as u16) | ((g2.min(31) as u16) << 5) | ((b2.min(31) as u16) << 10)
}
