mod win_info;

use super::window::get_windows;
use crate::{analyser::win_info::WinData, attach};
use eframe::{App, EventLoopBuilderHook};
use egui::{CentralPanel, Id, Layout, TopBottomPanel};
use std::{thread, time::Duration};
use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        UI::{
            Input::KeyboardAndMouse::{GetKeyState, GetKeyboardState},
            WindowsAndMessaging::{FindWindowA, GetCursorPos, GetWindowTextA},
        },
    },
    core::s,
};
use winit::platform::windows::EventLoopBuilderExtWindows;

pub fn analyse() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(1));

        let mut window_id = None;
        loop {
            window_id = unsafe { FindWindowA(None, s!("Worms Armageddon")) }.ok();
            if window_id.is_some() {
                break;
            }

            log::error!("Unable to find worms window");
            thread::sleep(Duration::from_secs(1));
        }
        let window_id = window_id.unwrap();

        let event_loop_builder: Option<EventLoopBuilderHook> =
            Some(Box::new(|event_loop_builder| {
                event_loop_builder.with_any_thread(true);
            }));
        let native_options = eframe::NativeOptions {
            event_loop_builder,
            ..Default::default()
        };

        eframe::run_native(
            "Window ider",
            native_options,
            Box::new(|cc| Ok(Box::new(MyApp::new(window_id)))),
        )
        .unwrap();
    });
}

#[derive(Default)]
struct MyApp {
    /// Window id of worms
    window_id: HWND,

    /// Only show the windows that contain the cursor.
    only_containing: bool,
    /// Show extra window information.
    show_info: bool,

    /// Window information
    window_data: Cache<10, WinData>,

    show_key_pressed: bool,
    key_index_modal: bool,
    keyboard_state: Cache<4, Vec<usize>>,
}

impl MyApp {
    fn new(window_id: HWND) -> Self {
        Self {
            window_id,
            only_containing: true,
            ..Default::default()
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut cursor_pos = POINT::default();
        unsafe { GetCursorPos(&mut cursor_pos as *mut POINT) };
        let cursor_pos = cursor_pos;

        let window_data = self
            .window_data
            .get(|| win_info::get_window_data(&get_windows(self.window_id)));

        if self.show_key_pressed {
            let pressed = self.keyboard_state.get(|| {
                let state = unsafe {
                    let mut state = [0u8; 256];
                    GetKeyboardState(&mut state);
                    state
                };

                state
                    .iter()
                    .enumerate()
                    .filter_map(|(index, key)| {
                        if *key & 0b1000_0000 == 0 {
                            return None;
                        }
                        Some(index)
                    })
                    .collect()
            });

            if pressed.len() != 0 || self.key_index_modal {
                self.key_index_modal = egui::Modal::new("grrr".into())
                    .show(ctx, |ui| {
                        ui.label(format!("Key: {pressed:?}"));
                    })
                    .should_close();
            }
        }

        TopBottomPanel::top("tap").show(ctx, |ui| {
            ui.label(format!("Cursor Position: {cursor_pos:?}"));

            ui.checkbox(&mut self.only_containing, "Only Show Containing Cursor");
            ui.checkbox(&mut self.show_info, "Show Long Info");

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Spawn Mod").clicked() {
                    attach();
                };
            })
        });

        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().animated(false).show(ui, |ui| {
                win_info::display(&window_data, ui, &cursor_pos, self.only_containing);
            });
        });

        ctx.request_repaint_after(Duration::from_millis(20));
    }
}

/// Stores cached data that is updated after `REFRESH_AFTER` accesses.
struct Cache<const REFRESH_AFTER: usize, T> {
    cache: Option<T>,
    accesses: usize,
}

// Manual impl because T doesn't require a Default impl.
impl<const REFRESH_AFTER: usize, T> Default for Cache<REFRESH_AFTER, T> {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            accesses: Default::default(),
        }
    }
}

impl<const REFRESH_AFTER: usize, T> Cache<REFRESH_AFTER, T> {
    /// If the data needs to be updated the given function is called.
    /// Otherwise the cached data is returned.
    fn get<Func: FnMut() -> T>(&mut self, mut func: Func) -> &mut T {
        if let (cache @ None, _) | (cache @ Some(_), true) =
            (&mut self.cache, self.accesses >= REFRESH_AFTER)
        {
            self.accesses = 0;
            *cache = Some(func());
        }

        self.accesses += 1;
        self.cache.as_mut().unwrap()
    }
}
