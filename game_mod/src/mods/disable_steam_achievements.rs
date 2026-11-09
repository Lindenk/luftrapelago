use crate::define_external_detour_hooks;

define_external_detour_hooks! {
  static SteamAPIInit from "steam_api.dll" => "SteamAPI_Init" : extern "C" fn() -> *const std::ffi::c_void;
}

pub fn disable_steam_achievements() {
  let _ = SteamAPIInitHook.init_with_self(move || std::ptr::null() as *const std::ffi::c_void);
  let _ = unsafe { SteamAPIInitHook.enable() };
}
