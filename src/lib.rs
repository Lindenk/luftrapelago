mod disable_saves;
mod fn_hooks;
mod sdl_fixes;
mod version_stub;

use std::ffi::c_void;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
  _h_module: HMODULE,
  ul_reason_for_call: u32,
  _lp_reserved: *mut c_void,
) -> bool {
  if ul_reason_for_call == DLL_PROCESS_ATTACH {
    std::thread::spawn(|| {
      unsafe {
        // Opens a visible terminal window alongside the game for debugging prints
        let _ = windows::Win32::System::Console::AllocConsole();
        println!("Loaded dll proxy");

        disable_saves::disable_saves();

        // Initialize window hooks (fixes alt-tab fullscreen crashes)
        //sdl_fixes::initialize_window_hooks();

        // Initialize your game logic / update hooks
        //initialize_game_hook();
      }
    });
  }
  true
}
