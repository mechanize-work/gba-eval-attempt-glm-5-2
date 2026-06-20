#!/usr/bin/env python3
"""Compare emulator wasm output against oracle reference."""
import subprocess
import struct
import sys
import os
import tempfile

WASM = "/task/target/wasm32-unknown-unknown/release/gba_emu.wasm"
ROM = sys.argv[1] if len(sys.argv) > 1 else "/task/dev-roms/anguna.gba"
FRAMES = int(sys.argv[2]) if len(sys.argv) > 2 else 60

def run_oracle(rom, frames):
    """Run oracle and return list of (framebuffer bytes, audio samples)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        audio_file = os.path.join(tmpdir, "audio.wav")
        frames_dir = os.path.join(tmpdir, "frames")
        result = subprocess.run(
            ["oracle", "run", rom, str(frames), 
             "--dump-frames", frames_dir,
             "--dump-audio", audio_file],
            capture_output=True, text=True
        )
        print(f"Oracle: {result.stdout.strip()}")
        
        frames_data = []
        for i in range(frames):
            ppm_path = os.path.join(frames_dir, f"frame_{i:05d}.ppm")
            if os.path.exists(ppm_path):
                with open(ppm_path, 'rb') as f:
                    ppm_data = f.read()
                # Parse PPM (P6 format)
                # Header: P6\n240 160\n255\n
                parts = ppm_data.split(b'\n', 3)
                width, height = map(int, parts[1].split())
                pixel_data = parts[3]
                # Convert RGB to ABGR
                fb = bytearray(width * height * 4)
                for j in range(width * height):
                    r = pixel_data[j * 3]
                    g = pixel_data[j * 3 + 1]
                    b = pixel_data[j * 3 + 2]
                    fb[j * 4] = r
                    fb[j * 4 + 1] = g
                    fb[j * 4 + 2] = b
                    fb[j * 4 + 3] = 0xFF
                frames_data.append(bytes(fb))
        
        return frames_data

def run_wasm_wasmtime(rom, frames):
    """Run our wasm emulator using wasmtime CLI."""
    # We need a host program that loads the wasm, calls the ABI functions.
    # Let's use a small Rust program or use wasmtime with a WAT wrapper.
    # Actually, let's use the wasmtime CLI with --invoke
    
    # Read ROM
    with open(rom, 'rb') as f:
        rom_data = f.read()
    
    # Write a simple host script that uses wasmtime Python bindings
    # Since we don't have Python wasmtime, let's write a Rust host
    pass

# For now, let's just get oracle frames and examine them
if __name__ == "__main__":
    frames = run_oracle(ROM, FRAMES)
    print(f"Got {len(frames)} oracle frames")
    
    # Save first frame as raw for inspection
    if frames:
        with open("/tmp/oracle_frame0.raw", 'wb') as f:
            f.write(frames[0])
        print(f"Saved first frame ({len(frames[0])} bytes)")
        
        # Check if frame is all black
        all_black = all(b == 0 for b in frames[0][::4])  # Check R channel
        print(f"Frame all black (R channel): {all_black}")
        
        # Print some pixel values
        for i in range(0, min(10, len(frames[0]) // 4)):
            r, g, b, a = frames[0][i*4], frames[0][i*4+1], frames[0][i*4+2], frames[0][i*4+3]
            print(f"  Pixel {i}: R={r} G={g} B={b} A={a}")
