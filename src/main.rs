extern crate libc;

use std::mem;
use std::ops::{Index, IndexMut};

extern {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

#[cfg(windows)]
mod memory {
    extern crate libc;
    extern crate winapi;
    extern crate kernel32;

    extern {
        fn _aligned_malloc(size: libc::size_t, alignment: libc::size_t) -> *mut libc::c_void;
    }

    pub unsafe fn aligned_malloc(size: libc::size_t, alignment: libc::size_t) -> *mut libc::c_void {
        _aligned_malloc(size, alignment)
    }

    pub unsafe fn make_executable(addr: *mut libc::c_void, size: libc::size_t) {
        let mut _previous_protect : *mut u32 = &mut 0u32 as *mut u32;
        kernel32::VirtualProtect(addr as *mut ::std::os::raw::c_void, size as u64, 
            winapi::winnt::PAGE_EXECUTE_READWRITE, _previous_protect as u32);
    }
}

#[cfg(unix)]
mod memory {
    extern crate libc;

    pub unsafe fn aligned_malloc(size: libc::size_t, alignment: libc::size_t) -> *mut libc::c_void {
        let mut _contents : *mut libc::c_void = ::std::ptr::null_mut();
        libc::posix_memalign(&mut _contents, alignment, size);

        _contents
    }

    pub unsafe fn make_executable(addr: *mut libc::c_void, size: libc::size_t) {
        libc::mprotect(addr, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
    }
}

const PAGE_SIZE: usize = 4096;

struct JitMemory {
    contents : *mut u8
}

impl JitMemory {
    fn new(num_pages: usize) -> JitMemory {
        let contents : *mut u8;
        unsafe {
            let size = num_pages * PAGE_SIZE;
            let mut _contents : *mut libc::c_void = memory::aligned_malloc(size, PAGE_SIZE);
            memory::make_executable(_contents, size);

            memset(_contents, 0xc3, size);  // for now, prepopulate with 'RET'

            contents = mem::transmute(_contents);
        }

        JitMemory { contents: contents }
    }
}

impl Index<usize> for JitMemory {
    type Output = u8;

    fn index(&self, _index: usize) -> &u8 {
        unsafe {&*self.contents.offset(_index as isize) }
    }
}

impl IndexMut<usize> for JitMemory {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        unsafe { &mut *self.contents.offset(_index as isize) }
    }
}

fn run_jit() -> (fn() -> i64) {
    let mut jit : JitMemory = JitMemory::new(1);

    jit[0] = 0x48;  // mov RAX, 0x3
    jit[1] = 0xc7;
    jit[2] = 0xc0;
    jit[3] = 0x3;
    jit[4] = 0;
    jit[5] = 0;
    jit[6] = 0;

    unsafe { mem::transmute(jit.contents) }
}

fn main() {
    let fun = run_jit();
    println!("{}", fun());
}
