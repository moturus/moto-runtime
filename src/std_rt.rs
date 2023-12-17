pub use moto_sys::syscalls::*;
pub use moto_sys::*;

pub fn num_cpus() -> u32 {
    moto_sys::num_cpus()
}

use core::alloc::GlobalAlloc;
use core::alloc::Layout;

struct BackEndAllocator {}

unsafe impl Send for BackEndAllocator {}
unsafe impl Sync for BackEndAllocator {}

unsafe impl GlobalAlloc for BackEndAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        const PAGE_4K: u64 = 1 << 12;
        assert_eq!(SysMem::PAGE_SIZE_SMALL, PAGE_4K);

        let alloc_size = align_up(layout.size() as u64, PAGE_4K);
        if let Ok(start) = SysMem::alloc(PAGE_4K, alloc_size >> 12) {
            start as usize as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        SysMem::free(ptr as usize as u64).unwrap()
    }
}

static BACK_END: BackEndAllocator = BackEndAllocator {};
// #[global_allocator]
static FRUSA: frusa::Frusa4K = frusa::Frusa4K::new(&BACK_END);

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    FRUSA.alloc(layout)
}

pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    FRUSA.alloc_zeroed(layout)
}

pub unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
    FRUSA.dealloc(ptr, layout)
}

pub unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    FRUSA.realloc(ptr, layout, new_size)
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn moturus_runtime_start() {}

#[cfg(not(test))]
pub fn moturus_start_rt() {
    moturus_runtime_start();
    if crate::fs::init().is_err() {
        crate::util::moturus_log!("Failed to initialize FS. Exiting.");
        exit(-1);
    }
}

pub fn sys_exit(code: u64) -> ! {
    SysCpu::exit(code)
}

pub fn exit(code: i32) -> ! {
    let code_u32: u32 = unsafe { core::mem::transmute::<i32, u32>(code) };
    sys_exit(code_u32 as u64)
}

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[no_mangle]
pub fn moturus_log_panic(info: &PanicInfo<'_>) {
    SysMem::log("PANIC").ok(); // Log w/o allocations.
    SysMem::log(alloc::format!("PANIC: {}", info).as_str()).ok();

    log_backtrace(crate::rt_api::process::binary().unwrap_or("<unknown>"));
}

#[cfg(not(test))]
#[panic_handler]
fn _panic(info: &PanicInfo<'_>) -> ! {
    moturus_log_panic(info);
    sys_exit(u64::MAX)
}

const BT_DEPTH: usize = 64;

fn get_backtrace() -> [u64; BT_DEPTH] {
    let mut backtrace: [u64; BT_DEPTH] = [0; BT_DEPTH];

    let mut rbp: u64;
    unsafe {
        core::arch::asm!(
            "mov rdx, rbp", out("rdx") rbp, options(nomem, nostack)
        )
    };

    if rbp == 0 {
        return backtrace;
    }

    // Skip the first stack frame, which is one of the log_backtrace
    // functions below.
    rbp = unsafe { *(rbp as *mut u64) };
    let mut prev = 0_u64;

    for idx in 0..BT_DEPTH {
        if prev == rbp {
            break;
        }
        if rbp == 0 {
            break;
        }
        if rbp < 1024 * 64 {
            break;
        }
        prev = rbp;
        unsafe {
            backtrace[idx] = *((rbp + 8) as *mut u64);
            rbp = *(rbp as *mut u64);
        }
    }

    backtrace
}

pub fn log_backtrace(binary: &str) {
    use core::fmt::Write;

    let mut writer = alloc::string::String::with_capacity(256);

    let backtrace = get_backtrace();
    write!(&mut writer, "backtrace: run \n\naddr2line -e {}", binary).ok();
    for addr in backtrace {
        if addr == 0 {
            break;
        }

        if addr > (1_u64 << 40) {
            break;
        }

        write!(&mut writer, " \\\n  0x{:x}", addr).ok();
    }

    write!(&mut writer, "\n\n").ok();

    SysMem::log(writer.as_str()).ok();
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn debug_alloc() -> bool {
    false
}
