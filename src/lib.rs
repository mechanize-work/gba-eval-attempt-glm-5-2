#![no_std]
#![no_main]

mod cpu;
mod memory;
mod ppu;
mod apu;
mod dma;
mod timer;
mod input;
mod interrupt;
mod emulator;

use core::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn emu_init() -> i32 {
    emulator::init();
    INITIALIZED.store(true, Ordering::SeqCst);
    1
}

#[no_mangle]
pub extern "C" fn emu_rom_buffer() -> *mut u8 {
    emulator::rom_buffer_ptr()
}

#[no_mangle]
pub extern "C" fn emu_load_rom(len: i32) -> i32 {
    if !INITIALIZED.load(Ordering::SeqCst) {
        return 0;
    }
    emulator::load_rom(len as usize)
}

#[no_mangle]
pub extern "C" fn emu_reset() -> i32 {
    if !INITIALIZED.load(Ordering::SeqCst) {
        return 0;
    }
    emulator::reset();
    1
}

#[no_mangle]
pub extern "C" fn emu_set_keys(k: u32) {
    emulator::set_keys(k);
}

#[no_mangle]
pub extern "C" fn emu_run_frame() {
    emulator::run_frame();
}

#[no_mangle]
pub extern "C" fn emu_framebuffer() -> *mut u32 {
    emulator::framebuffer_ptr()
}

#[no_mangle]
pub extern "C" fn emu_audio_buffer() -> *mut i16 {
    emulator::audio_buffer_ptr()
}

#[no_mangle]
pub extern "C" fn emu_audio_samples() -> i32 {
    emulator::audio_samples()
}

#[no_mangle]
pub extern "C" fn emu_audio_rate() -> i32 {
    emulator::audio_rate()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
