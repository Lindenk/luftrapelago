mod archipelago;
mod fn_hooks;
mod game_data;
mod mods;
mod patcher;
mod state;
mod util;
mod version_stub;

use std::ffi::c_void;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::archipelago::client::APClient;
use crate::mods::{
  disable_game_unlocks, disable_steam_achievements, disable_tutorial, draw_mission_hints,
  fix_hangar_screen, init_game_stats, redirect_saves,
};
use crate::state::{GAME_STATE, GameInitializing};

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
      let mut ap_client = APClient::with_console_prompt();

      // redirect the save file so we have a unique one per ap run
      let _ = unsafe { windows::Win32::System::Console::AllocConsole() };
      let seed = ap_client.client().seed_name();
      redirect_saves(seed);

      // disable steam achievements so we don't accidentally let the player
      // ruin their steam account
      disable_steam_achievements();

      // setup remaining hooks
      init_game_stats();
      disable_tutorial();
      disable_game_unlocks();
      fix_hangar_screen();
      draw_mission_hints();

      // update our state
      let mut state = GAME_STATE.lock().unwrap();
      *state = state::GameState::Intializing(GameInitializing { client: ap_client });
    }

    EntryHook.call()
  });
  let _ = unsafe { EntryHook.enable() };
}
