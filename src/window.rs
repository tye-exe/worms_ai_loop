use std::time::Duration;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{FindWindowExA, SendMessageA, WM_LBUTTONDOWN, WM_LBUTTONUP},
};

/// Gets all child windows from a parent window.
pub fn get_windows(parent: HWND) -> Found {
    let mut found = Found::new(parent);

    let mut child_id = None;

    loop {
        child_id = unsafe { FindWindowExA(Some(parent), child_id, None, None) }.ok();

        let Some(id) = child_id else {
            break;
        };

        let mut sub_found = Found::new(id);
        fill_windows(id, &mut sub_found);
        found.add_child(sub_found);
    }

    found
}

fn fill_windows(parent: HWND, found: &mut Found) {
    let mut child_id = None;

    loop {
        child_id = unsafe { FindWindowExA(Some(parent), child_id, None, None) }.ok();

        let Some(id) = child_id else {
            break;
        };

        let mut sub_found = Found::new(id);
        fill_windows(id, &mut sub_found);
        found.add_child(sub_found);
    }
}

/// The first value of the callback should return `Some(HWND)` for the desired window.
pub fn wait_for_window<ItrReturned: Iterator<Item = HWND>>(
    parent: HWND,
    callback: impl Fn(std::vec::IntoIter<HWND>) -> ItrReturned,
) -> HWND {
    loop {
        if let Some(id) = callback(get_windows(parent).into_iter()).next() {
            return id;
        };
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Recursively contains windows that are contained by a parent window.
pub struct Found {
    children: Vec<Found>,
    value: HWND,
}

impl Found {
    pub fn new(value: HWND) -> Self {
        Self {
            children: Default::default(),
            value,
        }
    }

    pub fn add_child(&mut self, found: Self) {
        self.children.push(found);
    }

    pub fn value(&self) -> HWND {
        self.value
    }

    pub fn children(&self) -> &[Found] {
        &self.children
    }

    pub fn get(&self, index: usize) -> Option<&Found> {
        self.children.get(index)
    }
}

impl Into<Vec<HWND>> for Found {
    fn into(self) -> Vec<HWND> {
        let Self { children, value } = self;

        let mut values = Vec::new();
        for child in children {
            values.append(&mut child.into());
        }
        values.push(value);
        values
    }
}

impl IntoIterator for Found {
    type Item = HWND;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Into::<Vec<HWND>>::into(self).into_iter()
    }
}

pub trait GetFound {
    fn get(&self, index: usize) -> Option<&Found>;

    fn value(&self) -> Option<HWND>;
}

impl GetFound for Option<&Found> {
    fn get(&self, index: usize) -> Option<&Found> {
        self.and_then(|found| found.get(index))
    }

    fn value(&self) -> Option<HWND> {
        self.map(|found| found.value())
    }
}

impl GetFound for Option<Found> {
    fn get(&self, index: usize) -> Option<&Found> {
        self.as_ref().and_then(|found| found.get(index))
    }

    fn value(&self) -> Option<HWND> {
        self.as_ref().map(|found| found.value())
    }
}

/// Provides a helper method to left click on a window.
pub trait Click {
    /// Performs a left click on this window.
    fn click(self);
}

impl Click for HWND {
    fn click(self) {
        // SAFETY, I'm passing the expected arguments to the function. The windows API docs did not mention any edge cases that i have to handle
        unsafe { SendMessageA(self, WM_LBUTTONDOWN, WPARAM(0 as usize), LPARAM(0isize)) };
        unsafe { SendMessageA(self, WM_LBUTTONUP, WPARAM(0 as usize), LPARAM(0isize)) };
    }
}

/// Provides a helper method to get the text from a window.
pub trait Text {
    /// Gets the text attribute for this window.
    /// This will return an empty string if the windows is not part of the current process.
    fn text(self) -> String;
}

impl Text for HWND {
    fn text(self) -> String {
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextA, GetWindowTextLengthA};

        let min_buff_len = unsafe { GetWindowTextLengthA(self) }.abs() /* It doesn't make sense for the text buffer to have a negative length */ as usize
            + 1; // The given length is 1 indexed

        let mut window_text = vec![0u8; min_buff_len];
        unsafe { GetWindowTextA(self, window_text.as_mut_slice()) };
        String::from_utf8(window_text.to_vec())
            .expect("Unable to get window text")
            .trim()
            .to_owned()
    }
}
