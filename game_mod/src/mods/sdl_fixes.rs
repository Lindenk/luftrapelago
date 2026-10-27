use detour::static_detour;
use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::UI::WindowsAndMessaging::{
  HMENU, WINDOW_EX_STYLE, WINDOW_STYLE, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
};

static_detour! {
    static CreateWindowExAHook: extern "system" fn(
        WINDOW_EX_STYLE, *const std::os::raw::c_char, *const std::os::raw::c_char, WINDOW_STYLE,
        i32, i32, i32, i32, HWND, HMENU, HINSTANCE, *mut c_void
    ) -> HWND;

    static CreateWindowExWHook: extern "system" fn(
        WINDOW_EX_STYLE, *const u16, *const u16, WINDOW_STYLE,
        i32, i32, i32, i32, HWND, HMENU, HINSTANCE, *mut c_void
    ) -> HWND;
}

pub unsafe fn initialize_window_hooks() {
  let user32_name = windows::core::s!("user32.dll");
  let user32_handle = match windows::Win32::System::LibraryLoader::GetModuleHandleA(user32_name) {
    Ok(handle) => handle,
    Err(_) => return,
  };

  // 1. Hook CreateWindowExA
  if let Some(addr) = windows::Win32::System::LibraryLoader::GetProcAddress(
    user32_handle,
    windows::core::s!("CreateWindowExA"),
  ) {
    let target_fn: extern "system" fn(
      WINDOW_EX_STYLE,
      *const std::os::raw::c_char,
      *const std::os::raw::c_char,
      WINDOW_STYLE,
      i32,
      i32,
      i32,
      i32,
      HWND,
      HMENU,
      HINSTANCE,
      *mut c_void,
    ) -> HWND = std::mem::transmute(addr);

    let _ = CreateWindowExAHook.initialize(
      target_fn,
      move |exstyle, classname, windowname, style, x, y, w, h, parent, menu, inst, param| {
        let is_fullscreen = (style & WS_POPUP).0 != 0 && (style & WS_OVERLAPPEDWINDOW).0 == 0;

        if is_fullscreen {
          let adjusted_style = WS_POPUP | WS_VISIBLE;
          CreateWindowExAHook.call(
            exstyle,
            classname,
            windowname,
            adjusted_style,
            x,
            y,
            w,
            h,
            parent,
            menu,
            inst,
            param,
          )
        } else {
          CreateWindowExAHook.call(
            exstyle, classname, windowname, style, x, y, w, h, parent, menu, inst, param,
          )
        }
      },
    );
    let _ = CreateWindowExAHook.enable();
  }

  // 2. Hook CreateWindowExW
  if let Some(addr) = windows::Win32::System::LibraryLoader::GetProcAddress(
    user32_handle,
    windows::core::s!("CreateWindowExW"),
  ) {
    let target_fn: extern "system" fn(
      WINDOW_EX_STYLE,
      *const u16,
      *const u16,
      WINDOW_STYLE,
      i32,
      i32,
      i32,
      i32,
      HWND,
      HMENU,
      HINSTANCE,
      *mut c_void,
    ) -> HWND = std::mem::transmute(addr);

    let _ = CreateWindowExWHook.initialize(
      target_fn,
      move |exstyle, classname, windowname, style, x, y, w, h, parent, menu, inst, param| {
        let is_fullscreen = (style & WS_POPUP).0 != 0 && (style & WS_OVERLAPPEDWINDOW).0 == 0;

        if is_fullscreen {
          let adjusted_style = WS_POPUP | WS_VISIBLE;
          // FIXED: Use CreateWindowExWHook here instead of AHook
          CreateWindowExWHook.call(
            exstyle,
            classname,
            windowname,
            adjusted_style,
            x,
            y,
            w,
            h,
            parent,
            menu,
            inst,
            param,
          )
        } else {
          CreateWindowExWHook.call(
            exstyle, classname, windowname, style, x, y, w, h, parent, menu, inst, param,
          )
        }
      },
    );
    let _ = CreateWindowExWHook.enable();
  }
}
