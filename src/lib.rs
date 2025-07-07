use windows::{core::s, Win32::{System::SystemServices::*, UI::WindowsAndMessaging::{MessageBoxA, MB_OK}}};

use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, };

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => (),
        _ => ()
    }

    true
}


fn attach() {
    unsafe {
        MessageBoxA(None, s!("Hello :3"), s!("It is beginning..."), MB_OK);
    }
}