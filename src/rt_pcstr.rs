//! See [pcstr]

/// Creates a [PCSTR][windows::core::PCSTR] at runtime, this allows for formatted strings.
#[macro_export]
macro_rules! pcstr {
    ($expr:expr) => {
        windows::core::PCSTR::from_raw(Box::new(::std::format!("{}{}", $expr, '\0')).as_ptr())
    };
}
