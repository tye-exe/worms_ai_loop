mod init;
mod rt_pcstr;

use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowA, FindWindowExA, GetClassNameA, GetWindowTextA,
};
use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};
use windows::core::s;

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            init::init();
            attach();
        }
        DLL_PROCESS_DETACH => log::info!("Detach time"),
        _ => (),
    }

    true
}

fn attach() {
    log::info!("Started");
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(3));
        let window_id = unsafe { FindWindowA(None, s!("Worms Armageddon")) }
            .expect("Unable to find `Worms Armageddon` window");
        log::info!("Window ID: {window_id:?}");
        find_child_windows(window_id);
    });
}

fn find_child_windows(parent: HWND) {
    let mut child_id = None;

    loop {
        child_id = unsafe { FindWindowExA(Some(parent), child_id, None, None) }.ok();

        let Some(id) = child_id else {
            break;
        };

        let mut text = [' ' as u8; 255];
        unsafe { GetWindowTextA(id, &mut text) };
        let text = String::from_utf8(text.to_vec()).expect("Unable to get window title");

        let mut name = [0u8; 255];
        unsafe { GetClassNameA(id, &mut name) };
        let name = String::from_utf8(name.to_vec()).expect("Unable to get window name");

        log::info!(
            "Child ID: {id:?}. Text: `{}`. Name: {}",
            text.trim(),
            name.trim()
        );

        find_child_windows(id);
    }
}
