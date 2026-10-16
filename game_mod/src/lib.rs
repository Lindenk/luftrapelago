mod archipelago;
mod fn_hooks;
mod game_data;
mod mods;
mod util;
mod version_stub;

use std::ffi::c_void;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::archipelago::client::Archipelago;
use crate::fn_hooks::EntryHook;
use crate::mods::disable_steam_achievements::disable_steam_achievements;
use crate::mods::redirect_saves::redirect_saves_patch;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
  _h_module: HMODULE,
  ul_reason_for_call: u32,
  _lp_reserved: *mut c_void,
) -> bool {
  if ul_reason_for_call == DLL_PROCESS_ATTACH {
    hook_entry();
  }
  true
}

pub fn hook_entry() {
  let _ = EntryHook.init_with_self(|| {
    let mut ap_client = Archipelago::with_console_prompt();

    // redirect the save file so we have a unique one per ap run
    let _ = unsafe { windows::Win32::System::Console::AllocConsole() };
    let seed = ap_client.client().seed_name();
    redirect_saves_patch(seed);

    // disable steam achievements so we don't accidentally let the player
    // ruin their steam account
    disable_steam_achievements();

    EntryHook.call()
  });
  let _ = unsafe { EntryHook.enable() };
}
