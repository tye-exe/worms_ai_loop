use std::time::Duration;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{FindWindowExA, GetClassNameA, GetWindowTextA},
};

pub fn get_windows(parent: HWND) -> Vec<HWND> {
    let mut windows = Vec::new();
    let mut child_id = None;

    loop {
        child_id = unsafe { FindWindowExA(Some(parent), child_id, None, None) }.ok();

        let Some(id) = child_id else {
            break;
        };

        windows.push(id);
        fill_windows(id, &mut windows);
    }

    windows
}

fn fill_windows(parent: HWND, windows: &mut Vec<HWND>) {
    let mut child_id = None;

    loop {
        child_id = unsafe { FindWindowExA(Some(parent), child_id, None, None) }.ok();

        let Some(id) = child_id else {
            break;
        };

        windows.push(id);
        fill_windows(id, windows);
    }
}

pub fn wait_for_window(parent: HWND, callback: impl Fn(Vec<HWND>) -> Option<HWND>) -> HWND {
    loop {
        if let Some(id) = callback(get_windows(parent)) {
            return id;
        };
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn log_child_windows(parent: HWND) {
    for id in get_windows(parent) {
        let mut text = [' ' as u8; 255];
        unsafe { GetWindowTextA(id, &mut text) };
        let text = String::from_utf8(text.to_vec()).expect("Unable to get window title");

        let mut name = [' ' as u8; 255];
        unsafe { GetClassNameA(id, &mut name) };
        let name = String::from_utf8(name.to_vec()).expect("Unable to get window name");

        log::info!(
            "Child ID: {id:?}. Text: `{}`. Name: {}",
            text.trim(),
            name.trim()
        );
    }
}
