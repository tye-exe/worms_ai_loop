mod analyser;
mod init;
mod rt_pcstr;
mod window;

use std::os::raw::c_void;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_OEM_1, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowA, FindWindowExA, GetClassNameA, GetWindowTextA, SendMessageA, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP,
};
use windows::core::s;
use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};

use crate::analyser::analyse;
use crate::window::{get_windows, log_child_windows, wait_for_window};

const CREATE_GAME_MENU: &str = "(1)Create single or multiplayer game";

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            init::init();
            //attach();
            analyse();
        }
        DLL_PROCESS_DETACH => log::info!("Detach time"),
        _ => (),
    }

    true
}

fn attach() {
    log::info!("Started");
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(1));
        let window_id = unsafe { FindWindowA(None, s!("Worms Armageddon")) }
            .expect("Unable to find `Worms Armageddon` window");
        log::info!("Window ID: {window_id:?}");

        log_child_windows(window_id);

        let id = wait_for_window(window_id, |windows| {
            windows
                .into_iter()
                .filter(|id| {
                    // Read window text into buffer
                    let mut window_text = [' ' as u8; 255];
                    unsafe { GetWindowTextA(*id, &mut window_text) };
                    let text = String::from_utf8(window_text.to_vec())
                        .expect("Unable to get window title");

                    text.starts_with(CREATE_GAME_MENU)
                })
                .next()
        });

        log::info!("Pressing: {id:?}");

        unsafe { SendMessageA(id, WM_LBUTTONDOWN, WPARAM(0 as usize), LPARAM(0isize)) };
        unsafe { SendMessageA(id, WM_LBUTTONUP, WPARAM(0 as usize), LPARAM(0isize)) };

        log_child_windows(window_id);
    });
}
