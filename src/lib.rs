mod analyser;
mod init;
mod window;

use mouce::{Mouse, MouseActions as _};
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowA, GetWindowRect, GetWindowTextA, SendMessageA, WHEEL_DELTA, WM_MOUSEWHEEL,
};
use windows::core::s;
use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};

use crate::analyser::analyse;
use crate::window::{Click as _, GetFound, Text as _, get_windows, wait_for_window};

const CREATE_GAME_MENU: &str = "(1)Create single or multiplayer game";
const ADD_TEAM_MESSAGE: &str = "Left click a team to add it to the game. Right click to edit.";
const ROUND_RESULTS_TEXT: &str = "(1) ROUND RESULTS";

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            init::init();
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

        let id = wait_for_window(window_id, |windows| {
            windows.filter(|id| {
                // Read window text into buffer
                let mut window_text = [' ' as u8; 255];
                unsafe { GetWindowTextA(*id, &mut window_text) };
                let text =
                    String::from_utf8(window_text.to_vec()).expect("Unable to get window title");

                text.starts_with(CREATE_GAME_MENU)
            })
        });

        log::info!("Pressing: {id:?}");

        // Select multiplayer game
        id.click();

        let mut button = get_windows(window_id).get(0).get(38).get(1).value();

        while button.is_none() {
            button = get_windows(window_id).get(0).get(38).get(1).value();
            std::thread::sleep(Duration::from_millis(200));
        }
        let button = button.unwrap();
        log::info!("found button");

        // Scroll to bottom of the teams list
        unsafe {
            SendMessageA(
                button,
                WM_MOUSEWHEEL,
                // The multiplier controls the scroll strength
                WPARAM(((WHEEL_DELTA as i64 * -5) << 16) as usize),
                LPARAM(0isize),
            )
        };

        add_teams(window_id, 2);

        // Click on play button
        log::debug!("Clicking play button");
        get_windows(window_id)
            .get(0)
            .get(2)
            .value()
            .expect("Unable to find play button")
            .click();

        // Wait until button after match is shown
        log::debug!("Waiting until round finishes");
        let mut round_results = None;
        while round_results.is_none() {
            round_results = get_windows(window_id)
                .get(0)
                .get(5)
                .value()
                .filter(|found| found.text().contains(ROUND_RESULTS_TEXT));
            std::thread::sleep(Duration::from_millis(250));
        }
        log::debug!("Round finished");

        // Exit the round over screen
        get_windows(window_id)
            .get(0)
            .get(1)
            .value()
            .expect("Unable to get exit button")
            .click();
    });
}

/// When on the round configuration, adds the given number of teams via using the mouse cursor.
fn add_teams(armageddon_id: HWND, num_teams: u8) {
    // Gets the position of the bar under the window for mouse position
    let add_team = get_windows(armageddon_id)
        .get(0)
        .get(35)
        .value()
        .expect("Unable to get add teams button");

    let mut place = RECT::default();
    unsafe {
        GetWindowRect(add_team, &mut place as *mut RECT).expect("Able to get window position");
    }

    let x_middle = place.left + ((place.right - place.left) / 2);
    let mut y_pos = place.top;
    let mouse = Mouse::new();
    let mut previous_position: Option<(i32, i32)> = None;

    mouse.move_to(x_middle, y_pos).expect("Able to move mouse");

    for _ in 0..num_teams {
        loop {
            // Check if mouse moved manually
            {
                if let Some(prev_pos) = previous_position
                    && prev_pos.1 - 2
                        != mouse
                            .get_position()
                            .expect("Unable to get mouse position")
                            .1
                {
                    log::info!("Manual mouse movement detected; sleeping");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }

                previous_position =
                    Some(mouse.get_position().expect("Unable to get mouse position"));
            }

            // Move mouse until the mouse is over a team
            y_pos -= 2;
            mouse.move_to(x_middle, y_pos).expect("Able to move mouse");

            let text = get_windows(armageddon_id)
                .get(0)
                .get(9)
                .value()
                .expect("Unable to get message box")
                .text();

            if text.contains(ADD_TEAM_MESSAGE) {
                mouse
                    .click_button(mouce::common::MouseButton::Left)
                    .expect("Able to click mouse button");

                std::thread::sleep(Duration::from_millis(100)); // Otherwise it is too fast
                break;
            }
        }
    }
}
