mod win_info;

use super::window::get_windows;
use crate::attach;
use eframe::{App, EventLoopBuilderHook};
use egui::{CentralPanel, Id, Layout, TopBottomPanel};
use std::{thread, time::Duration};
use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        UI::WindowsAndMessaging::{FindWindowA, GetCursorPos, GetWindowTextA},
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

struct MyApp {
    /// Window id of worms
    window_id: HWND,

    /// Only show the windows that contain the cursor.
    only_containing: bool,
    /// Show extra window information.
    show_info: bool,

    cached_data: Option<win_info::WinData>,
    tick: usize,
}

impl MyApp {
    fn new(window_id: HWND) -> Self {
        Self {
            window_id,
            only_containing: true,
            show_info: false,
            cached_data: None,
            tick: 0,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut cursor_pos = POINT::default();
        unsafe { GetCursorPos(&mut cursor_pos as *mut POINT) };
        let cursor_pos = cursor_pos;

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
            let window_data = match (&self.cached_data, self.tick) {
                (None, t) if t % 10 == 0 => {
                    &win_info::get_window_data(&get_windows(self.window_id))
                }
                (Some(var), ..) => var,
                _ => &win_info::get_window_data(&get_windows(self.window_id)),
            };

            egui::ScrollArea::vertical().animated(false).show(ui, |ui| {
                win_info::display(&window_data, ui, &cursor_pos, self.only_containing);
            });
        });

        ctx.request_repaint_after(Duration::from_millis(20));
        self.tick = self.tick.overflowing_add(1).0;
    }
}
