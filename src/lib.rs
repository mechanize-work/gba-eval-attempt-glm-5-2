#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

// Minimal global allocator for wasm32-unknown-unknown
// Uses wasm memory.grow to allocate pages
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

const WASM_PAGE_SIZE: usize = 65536;

struct WasmAllocator {
    heap_ptr: UnsafeCell<usize>,
}

static ALLOC: WasmAllocator = WasmAllocator {
    heap_ptr: UnsafeCell::new(0),
};

unsafe impl Sync for WasmAllocator {}

impl WasmAllocator {
    fn heap_ptr(&self) -> usize {
        unsafe { *self.heap_ptr.get() }
    }

    fn set_heap_ptr(&self, val: usize) {
        unsafe { *self.heap_ptr.get() = val; }
    }
}

// No extern needed - use core::arch::wasm32 intrinsics

unsafe impl GlobalAlloc for WasmAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let mut current = self.heap_ptr();
        if current == 0 {
            // Initialize: start after the data segment
            // We'll start at page 1 to be safe
            current = WASM_PAGE_SIZE;
            self.set_heap_ptr(current);
        }

        // Align
        let aligned = (current + align - 1) & !(align - 1);
        let new_ptr = aligned + size;

        // Check if we need more pages
        let current_pages = core::arch::wasm32::memory_size(0) * WASM_PAGE_SIZE;
        if new_ptr > current_pages {
            let needed_pages = (new_ptr - current_pages + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE;
            let result = core::arch::wasm32::memory_grow(0, needed_pages);
            if result == usize::MAX {
                return ptr::null_mut();
            }
        }

        self.set_heap_ptr(new_ptr);
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Simple bump allocator - no deallocation
    }
}

#[global_allocator]
static A: WasmAllocator = WasmAllocator {
    heap_ptr: UnsafeCell::new(0),
};

mod cpu;
mod cpu_arm;
mod cpu_thumb;
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
