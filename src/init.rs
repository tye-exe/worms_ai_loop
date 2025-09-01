use windows::{
    Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxA},
    core::PCSTR,
};

const LOG_FILE: &str = "worms_ai_loop.log";

/// Sets up:
/// - A custom panic hook to display in a message box
/// - Logging
pub fn init() {
    std::panic::set_hook(Box::new(|info| unsafe {
        log::error!(
            "Line: {}\nError: {}",
            info.location()
                .map(|a| a.to_string())
                .unwrap_or("Unknown Location".to_owned()),
            get_payload(info)
        );

        // Has to be in a separate variable to work
        let caption = concat!("Unrecoverable error : ", std::env!("CARGO_PKG_NAME"), "\0");
        let caption = PCSTR(caption.as_ptr());

        // Has to be in a separate variable to work
        let text = format!(
            "Line: {}\nError: {}\0",
            info.location()
                .map(|a| a.to_string())
                .unwrap_or("Unknown Location".to_owned()),
            get_payload(info)
        );
        let text = PCSTR(text.as_ptr());

        MessageBoxA(None, text, caption, MB_ICONERROR);

        std::process::exit(1);
    }));

    simple_logging::log_to_file(LOG_FILE, log::LevelFilter::Info)
        .expect("Unable to create log file");
}

/// See 'payload' in [std::panic::PanicHookInfo]
fn get_payload<'a>(info: &'a std::panic::PanicHookInfo<'_>) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_owned().to_owned()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.to_owned()
    } else {
        "Unknown reason".to_owned()
    }
}
