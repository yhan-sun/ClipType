//! Disposable P1-S01 Windows mechanism probe.
//!
//! This example validates thread-message-queue ownership and hotkey registration
//! without reading clipboard text or emitting synthetic input. It is research
//! evidence, not a production adapter.

#[cfg(not(windows))]
fn main() {
    println!("p1_s01_windows_native: skipped (non-Windows host)");
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_probe::run() {
        eprintln!("p1_s01_windows_native: failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod windows_probe {
    use core::ffi::c_void;
    use std::{mem::zeroed, ptr::null_mut, thread, time::Duration};

    const MOD_ALT: u32 = 0x0001;
    const MOD_CONTROL: u32 = 0x0002;
    const MOD_SHIFT: u32 = 0x0004;
    const MOD_NOREPEAT: u32 = 0x4000;
    const VK_F24: u32 = 0x87;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    const VK_LWIN: i32 = 0x5B;
    const VK_RWIN: i32 = 0x5C;
    const PM_NOREMOVE: u32 = 0x0000;
    const WM_APP: u32 = 0x8000;
    const PROBE_MESSAGE: u32 = WM_APP + 0x43;
    const HOTKEY_ID: i32 = 0x4354;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Message {
        hwnd: *mut c_void,
        message: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        point: Point,
        private: u32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        #[link_name = "RegisterHotKey"]
        fn register_hot_key(hwnd: *mut c_void, id: i32, modifiers: u32, key: u32) -> i32;

        #[link_name = "UnregisterHotKey"]
        fn unregister_hot_key(hwnd: *mut c_void, id: i32) -> i32;

        #[link_name = "PeekMessageW"]
        fn peek_message_w(
            message: *mut Message,
            hwnd: *mut c_void,
            filter_min: u32,
            filter_max: u32,
            remove_message: u32,
        ) -> i32;

        #[link_name = "GetMessageW"]
        fn get_message_w(
            message: *mut Message,
            hwnd: *mut c_void,
            filter_min: u32,
            filter_max: u32,
        ) -> i32;

        #[link_name = "PostThreadMessageW"]
        fn post_thread_message_w(
            thread_id: u32,
            message: u32,
            w_param: usize,
            l_param: isize,
        ) -> i32;

        #[link_name = "GetAsyncKeyState"]
        fn get_async_key_state(key: i32) -> i16;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetCurrentThreadId"]
        fn get_current_thread_id() -> u32;
    }

    pub fn run() -> Result<(), String> {
        let mut ignored: Message = unsafe { zeroed() };

        // Calling PeekMessage creates the current thread's message queue even
        // when no message is available. The probe never inspects user input.
        let _ = unsafe {
            peek_message_w(
                &mut ignored,
                null_mut(),
                PROBE_MESSAGE,
                PROBE_MESSAGE,
                PM_NOREMOVE,
            )
        };

        let registered = unsafe {
            register_hot_key(
                null_mut(),
                HOTKEY_ID,
                MOD_CONTROL | MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
                VK_F24,
            )
        };
        if registered == 0 {
            return Err("thread-owned development hotkey registration failed".into());
        }

        let owner_thread = unsafe { get_current_thread_id() };
        let worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(25));
            let posted = unsafe { post_thread_message_w(owner_thread, PROBE_MESSAGE, 0, 0) };
            (posted != 0)
                .then_some(())
                .ok_or_else(|| "worker could not signal the message-loop owner".to_string())
        });

        let mut message: Message = unsafe { zeroed() };
        let received = unsafe {
            get_message_w(&mut message, null_mut(), PROBE_MESSAGE, PROBE_MESSAGE)
        };

        let unregistered = unsafe { unregister_hot_key(null_mut(), HOTKEY_ID) };
        let worker_result = worker
            .join()
            .map_err(|_| "probe worker panicked".to_string())?;
        worker_result?;

        if received <= 0 || message.message != PROBE_MESSAGE {
            return Err("message-loop owner did not receive the worker signal".into());
        }
        if unregistered == 0 {
            return Err("development hotkey teardown failed".into());
        }

        let modifier_sample = [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN]
            .map(|key| unsafe { get_async_key_state(key) })
            .map(|state| state < 0);
        let held_modifier_count = modifier_sample.into_iter().filter(|held| *held).count();

        println!(
            "p1_s01_windows_native: hotkey_registration=ok message_loop_signal=ok teardown=ok held_modifier_count={held_modifier_count}"
        );
        Ok(())
    }
}
