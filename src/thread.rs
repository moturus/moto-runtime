use moto_sys::syscalls::*;
use moto_sys::ErrorCode;

pub fn spawn(
    stack_size: usize,
    thread_fn: usize,
    thread_arg: usize,
) -> Result<SysHandle, ErrorCode> {
    SysCpu::spawn(
        SysHandle::SELF,
        stack_size as u64,
        thread_fn as u64,
        thread_arg as u64,
    )
}

pub fn exit_self() -> ! {
    let _ = SysCtl::put(SysHandle::SELF);
    unreachable!()
}

pub fn join(handle: SysHandle) {
    // wait() below will succeed if it is called while the thread is still running
    // and will fail if the thread has exited.
    let _ = SysCpu::wait(&mut [handle], SysHandle::NONE, SysHandle::NONE, None);
}

pub fn sleep(dur: core::time::Duration) {
    // The current thread may have pending wakeups, and so the wait will immediately
    // return. Thus we have to track time and wait again.
    let stop = moto_sys::time::Instant::now() + dur;
    loop {
        match SysCpu::wait(&mut [], SysHandle::NONE, SysHandle::NONE, Some(stop)) {
            Ok(()) => continue,
            Err(ErrorCode::TimedOut) => {
                debug_assert!(moto_sys::time::Instant::now() >= stop);
                return;
            }
            Err(err) => {
                panic!("Unrecognized error code: {:?}", err);
            }
        }
    }
}
