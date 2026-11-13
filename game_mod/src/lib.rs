mod archipelago;
mod fn_hook;
mod game_data;
mod memory_matcher;
mod mods;
mod patcher;
mod state;
mod util;
mod version_stub;

use std::ffi::c_void;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::archipelago::client::APClient;
use crate::mods::*;

define_detour_hooks! {
  static Entry: extern "C" fn() -> () => 0x1dc925;
}

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
    {
      // connect to the ap server (make sure this works before we start up, even
      // if we disconnect after)
      let mut ap_client = APClient::with_console_prompt();
      let seed = ap_client.client().seed_name().to_string();
      let player = ap_client.client().this_player().to_string();

      // enable debug console
      #[cfg(debug_assertions)]
      let _ = unsafe { windows::Win32::System::Console::AllocConsole() };

      // the client should be connected here. set up our mods and hooks
      redirect_saves(format!("{}_{}", player, seed).as_str());
      init_mod(ap_client);
      hook_update_tick();

      disable_game_unlocks();
      disable_steam_achievements();
      disable_tutorial();

      fix_hangar_screen();
      draw_mission_hints();
      apply_powerups_and_traps();
      apply_player_mods();
      hook_on_die();
      set_up_rank_keys();
      display_completions();
    }

    EntryHook.call()
  });
  let _ = unsafe { EntryHook.enable() };
}
