use super::window::{Found, get_windows};
use crate::attach;
use eframe::{App, EventLoopBuilderHook};
use egui::{CentralPanel, Color32, Id, Layout, TopBottomPanel};
use std::{thread, time::Duration};
use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        UI::WindowsAndMessaging::{
            FindWindowA, GET_CLASS_LONG_INDEX, GetClassLongA, GetClassNameA, GetCursorPos,
            GetWindowInfo, GetWindowTextA, GetWindowTextW, RealGetWindowClassA, WINDOWINFO,
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

struct MyApp {
    /// Window id of worms
    window_id: HWND,
    id: u64,

    /// Only show the windows that contain the cursor.
    only_containing: bool,
    /// Show extra window information.
    show_info: bool,

    cached_data: Option<Found>,
    tick: usize,
}

impl MyApp {
    fn new(window_id: HWND) -> Self {
        Self {
            window_id,
            id: 1,
            only_containing: true,
            show_info: false,
            cached_data: None,
            tick: 0,
        }
    }

    fn get_id(&mut self) -> Id {
        let id = Id::new(self.id);
        self.id = self.id.wrapping_add(1);
        if self.id == 0 {
            self.id = 1;
        }
        id
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
            let found = match (&self.cached_data, self.tick) {
                (None, t) if t % 10 == 0 => &get_windows(self.window_id),
                (Some(var), ..) => var,
                _ => &get_windows(self.window_id),
            };

            egui::ScrollArea::vertical().animated(false).show(ui, |ui| {
                for_windows(found, 0, 0, Vec::default(), &mut |data, tree_position| {
                    let pos = data.info.rcWindow;

                    // If window contains cursor
                    if pos.bottom > cursor_pos.y
                        && pos.top < cursor_pos.y
                        && pos.left < cursor_pos.x
                        && pos.right > cursor_pos.x
                    {
                        ui.visuals_mut().override_text_color = Some(Color32::LIGHT_RED);
                    } else if self.only_containing {
                        return;
                    }

                    ui.separator();
                    let response = ui
                        .horizontal(|ui| {
                            ui.add_space((tree_position.depth * 50) as f32);
                            ui.vertical(|ui| {
                                ui.label(format!("Name: '{}'", data.name));
                                ui.label(format!("Text: '{}'", data.text));
                                ui.label(format!("Path: '{:?}'", tree_position.path));
                                // ui.label(format!(
                                //     "Depth: {}, Index: {}",
                                //     tree_position.depth, tree_position.index
                                // ));
                                //ui.label(format!("Real Class: '{}'", data.w_type));
                                //ui.label(format!("Atom: '{}'", data.atom));
                                if self.show_info {
                                    ui.label(format!("Info: {:#?}", data.info));
                                }
                            });
                        })
                        .response;

                    if self.only_containing {
                        ui.scroll_to_rect(response.rect, Some(egui::Align::TOP));
                    }

                    ui.visuals_mut().override_text_color = None;
                });
            });
        });

        ctx.request_repaint_after(Duration::from_millis(20));
        //ctx.request_repaint();
        self.tick = self.tick.overflowing_add(1).0;
    }
}

/// Data about each window.
struct WindowData<'a> {
    text: &'a str,
    name: &'a str,
    w_type: &'a str,
    info: WINDOWINFO,
    atom: u32,
}

struct Position {
    pub depth: usize,
    pub index: usize,
    pub path: Vec<usize>,
}

/// Executes the callback for each window that has been found.
///
/// This is a pre-order traversal.
fn for_windows(
    found: &Found,
    depth: usize,
    index: usize,
    path: Vec<usize>,
    callback: &mut impl FnMut(WindowData, Position) -> (),
) {
    let id = found.value();
    let mut text = [' ' as u16; 255];
    unsafe { GetWindowTextW(id, &mut text) };
    let text = String::from_utf16(&text).expect("Unable to get window title");

    let mut name = [' ' as u8; 255];
    unsafe { GetClassNameA(id, &mut name) };
    let name = String::from_utf8(name.to_vec()).expect("Unable to get window name");

    let mut w_type = [' ' as u8; 255];
    unsafe { RealGetWindowClassA(id, &mut w_type) };
    let w_type = String::from_utf8(w_type.to_vec()).expect("Unable to get window type");

    let mut info = WINDOWINFO::default();
    unsafe { GetWindowInfo(id, &mut info as *mut WINDOWINFO) };

    let atom = unsafe { GetClassLongA(id, GET_CLASS_LONG_INDEX(-32)) };

    callback(
        WindowData {
            text: text.trim(),
            name: name.trim(),
            w_type: w_type.trim(),
            info,
            atom,
        },
        Position {
            depth,
            index,
            path: path.clone(),
        },
    );

    for (index, child) in found.children().iter().enumerate() {
        for_windows(
            child,
            depth + 1,
            index,
            {
                let mut path = path.clone();
                path.push(index);
                path
            },
            callback,
        );
    }
}
