use super::super::window::Found;
use egui::{Color32, Ui};
use windows::Win32::{
    Foundation::POINT,
    UI::WindowsAndMessaging::{
        GET_CLASS_LONG_INDEX, GetClassLongA, GetClassNameA, GetWindowInfo, GetWindowTextW,
        RealGetWindowClassA, WINDOWINFO,
    },
};

/// Display window information to a egui ui.
pub(crate) fn display(
    data: &WinData,
    ui: &mut Ui,
    cursor_pos: &POINT,
    only_containing: bool,
    show_extra_info: bool,
) {
    let pos = data.info.rcWindow;

    // If window contains cursor
    if pos.bottom > cursor_pos.y
        && pos.top < cursor_pos.y
        && pos.left < cursor_pos.x
        && pos.right > cursor_pos.x
    {
        ui.visuals_mut().override_text_color = Some(Color32::LIGHT_RED);
    } else if only_containing {
        return;
    }

    ui.separator();
    let response = ui
        .horizontal(|ui| {
            ui.add_space((data.depth * 50) as f32);
            ui.vertical(|ui| {
                ui.label(format!("Name: '{}'", data.name));
                ui.label(format!("Text: '{}'", data.text));
                ui.label(format!("Path: '{:?}'", data.path));
                if show_extra_info {
                    ui.label(format!("Info: {:#?}", data.info));
                }
            });
        })
        .response;

    if only_containing {
        ui.scroll_to_rect(response.rect, Some(egui::Align::TOP));
    }

    ui.visuals_mut().override_text_color = None;

    for data in data.children.iter() {
        display(&data, ui, cursor_pos, only_containing, show_extra_info);
    }
}

/// Contains information about a window
pub(crate) struct WinData {
    pub(crate) text: Box<str>,
    pub(crate) name: Box<str>,
    pub(crate) w_type: Box<str>,
    pub(crate) info: WINDOWINFO,
    pub(crate) atom: u32,

    pub(crate) depth: usize,
    pub(crate) path: Vec<usize>,

    pub(crate) children: Vec<WinData>,
}

impl WinData {
    pub(crate) fn new(
        text: impl Into<Box<str>>,
        name: impl Into<Box<str>>,
        w_type: impl Into<Box<str>>,
        info: WINDOWINFO,
        atom: u32,
        depth: usize,
        path: Vec<usize>,
    ) -> Self {
        Self {
            text: text.into(),
            name: name.into(),
            w_type: w_type.into(),
            info,
            atom,
            children: Vec::new(),
            depth,
            path,
        }
    }

    pub(crate) fn add_child(&mut self, child: WinData) {
        self.children.push(child);
    }

    pub fn last_child_containing(&self, cursor_pos: &POINT) -> Option<&WinData> {
        let pos = self.info.rcWindow;

        // If window does not contain cursor
        if !(pos.bottom > cursor_pos.y
            && pos.top < cursor_pos.y
            && pos.left < cursor_pos.x
            && pos.right > cursor_pos.x)
        {
            return None;
        }

        for data in self.children.iter() {
            if let Some(contains) = data.last_child_containing(cursor_pos) {
                return Some(contains);
            };
        }

        return Some(self);
    }
}

/// Gets the [`WinData`] for found and all child windows.
pub(crate) fn get_window_data(found: &Found) -> WinData {
    get_window_data_rec(found, 0, Vec::new())
}

fn get_window_data_rec(found: &Found, depth: usize, path: Vec<usize>) -> WinData {
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

    let mut data = WinData::new(
        text.trim(),
        name.trim(),
        w_type.trim(),
        info,
        atom,
        depth,
        path.clone(),
    );

    for (index, child) in found.children().iter().enumerate() {
        data.add_child(get_window_data_rec(child, depth + 1, {
            let mut v = path.clone();
            v.push(index);
            v
        }));
    }

    data
}
