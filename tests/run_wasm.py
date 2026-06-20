#!/usr/bin/env python3
"""Generate a WAT host module that drives the GBA emulator and outputs framebuffer."""
import sys
import struct
import os

def generate_wat(rom_path, frames, output_fb=True, output_audio=True):
    with open(rom_path, 'rb') as f:
        rom_data = f.read()
    
    rom_len = len(rom_data)
    
    # Generate data segments for ROM
    # We'll write the ROM data as a data segment at a known offset in the shared memory
    # The emu_rom_buffer() returns a pointer to the ROM buffer in the wasm memory
    # We need to: 1) call emu_init, 2) call emu_rom_buffer to get ptr, 
    # 3) write ROM data to that ptr, 4) call emu_load_rom
    
    # But in WAT, we can't easily do dynamic memory writes with data segments
    # because we don't know the pointer at compile time.
    
    # Alternative approach: write ROM data to a fixed offset (e.g., 0x10000)
    # then copy it to rom_buffer in the _start function
    
    # Actually, let me use a different approach: write the ROM data starting at 
    # a high memory address that won't conflict, then use memcopy in WAT
    
    # Simplest: embed ROM as hex data in the WAT, use i32.store to write each byte
    # But that's very large for multi-MB ROMs
    
    # Better: use data segments. We can put data at a fixed address,
    # then in _start, call emu_rom_buffer(), and copy from fixed address to that ptr
    
    # Let's put ROM at address 0x40000 (256KB offset - should be safe)
    rom_offset = 0x40000
    
    lines = []
    lines.append('(module')
    lines.append('  ;; Import GBA emulator functions')
    lines.append('  (import "gba_emu" "emu_init" (func $emu_init (result i32)))')
    lines.append('  (import "gba_emu" "emu_rom_buffer" (func $emu_rom_buffer (result i32)))')
    lines.append('  (import "gba_emu" "emu_load_rom" (func $emu_load_rom (param i32) (result i32)))')
    lines.append('  (import "gba_emu" "emu_reset" (func $emu_reset (result i32)))')
    lines.append('  (import "gba_emu" "emu_set_keys" (func $emu_set_keys (param i32)))')
    lines.append('  (import "gba_emu" "emu_run_frame" (func $emu_run_frame)))')
    lines.append('  (import "gba_emu" "emu_framebuffer" (func $emu_framebuffer (result i32)))')
    lines.append('  (import "gba_emu" "emu_audio_buffer" (func $emu_audio_buffer (result i32)))')
    lines.append('  (import "gba_emu" "emu_audio_samples" (func $emu_audio_samples (result i32)))')
    lines.append('  (import "gba_emu" "emu_audio_rate" (func $emu_audio_rate (result i32)))')
    lines.append('  (import "gba_emu" "memory" (memory 1))')
    lines.append('')
    
    # ROM data as a data segment
    # Write ROM data as binary data segment
    lines.append(f'  ;; ROM data ({rom_len} bytes) at offset 0x{rom_offset:X}')
    
    # Write data in chunks
    chunk_size = 4096
    for offset in range(0, rom_len, chunk_size):
        chunk = rom_data[offset:offset+chunk_size]
        hex_str = ''.join(f'\\{b:02x}' for b in chunk)
        lines.append(f'  (data (i32.const {rom_offset + offset}) "{hex_str}")')
    
    lines.append('')
    
    # Memory copy function
    lines.append('  (func $memcpy (param $dst i32) (param $src i32) (param $len i32)')
    lines.append('    (local $i i32)')
    lines.append('    (local.set $i (i32.const 0))')
    lines.append('    (block $break')
    lines.append('      (loop $loop')
    lines.append('        (br_if $break (i32.ge_s (local.get $i) (local.get $len)))')
    lines.append('        (i32.store8')
    lines.append('          (i32.add (local.get $dst) (local.get $i))')
    lines.append('          (i32.load8_u (i32.add (local.get $src) (local.get $i))))')
    lines.append('        (local.set $i (i32.add (local.get $i) (i32.const 1)))')
    lines.append('        (br $loop)))')
    lines.append('  )')
    lines.append('')
    
    # _start function
    lines.append('  (func (export "_start")')
    lines.append('    (local $rom_ptr i32)')
    lines.append('    (local $fb_ptr i32)')
    lines.append('    (local $i i32)')
    lines.append('    (local $frame i32)')
    lines.append('')
    lines.append('    ;; Initialize emulator')
    lines.append('    (call $emu_init)')
    lines.append('    drop')
    lines.append('')
    lines.append('    ;; Get ROM buffer pointer')
    lines.append('    (local.set $rom_ptr (call $emu_rom_buffer))')
    lines.append('')
    lines.append(f'    ;; Copy ROM data ({rom_len} bytes)')
    lines.append(f'    (call $memcpy (local.get $rom_ptr) (i32.const {rom_offset}) (i32.const {rom_len}))')
    lines.append('')
    lines.append(f'    ;; Load ROM')
    lines.append(f'    (call $emu_load_rom (i32.const {rom_len}))')
    lines.append('    drop')
    lines.append('')
    
    # Run frames and output framebuffer
    for frame in range(frames):
        lines.append(f'    ;; Frame {frame}')
        lines.append(f'    (call $emu_run_frame)')
        if output_fb and frame == frames - 1:
            lines.append(f'    ;; Get framebuffer pointer for last frame')
            lines.append(f'    (local.set $fb_ptr (call $emu_framebuffer))')
            lines.append(f'    ;; Output framebuffer to stdout (153600 bytes)')
            lines.append(f'    (local.set $i (i32.const 0))')
            lines.append(f'    (block $fb_break_{frame}')
            lines.append(f'      (loop $fb_loop_{frame}')
            lines.append(f'        (br_if $fb_break_{frame} (i32.ge_s (local.get $i) (i32.const 153600)))')
            lines.append(f'        (call $write_byte')
            lines.append(f'          (i32.load8_u (i32.add (local.get $fb_ptr) (local.get $i))))')
            lines.append(f'        (local.set $i (i32.add (local.get $i) (i32.const 1)))')
            lines.append(f'        (br $fb_loop_{frame})))')
        lines.append('')
    
    lines.append('  )')
    lines.append('')
    
    # Import write_byte from WASI
    # Actually, we need WASI for fd_write. Let me use a simpler approach.
    # We can use the WASI fd_write to write to stdout.
    
    # Actually, let me just output the framebuffer pointer and audio info
    # as return values, and read the memory externally.
    
    lines.append(')')
    
    return '\n'.join(lines)


# Actually, the WAT approach is getting too complex. Let me try a completely different approach.
# I'll write a C test harness that uses wasm3 or a minimal wasm interpreter.
# Or better yet, let me compile our emulator for native target (x86_64) and test it directly!

print("Generating native test harness instead...")
