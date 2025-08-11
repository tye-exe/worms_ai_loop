use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::FindWindowExA};

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
}
