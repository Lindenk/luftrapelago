use crate::{fn_hooks::SteamUserStatsHook, util::patch_memory};

// Define our custom stub function with the correct calling convention (__thiscall / extern "C" with explicit this pointer)
// Steamworks SDK functions use __thiscall on Windows, which passes the interface pointer (`this`) in ECX.
// In Rust FFI, we declare this as taking the raw `this` pointer as its first argument.
unsafe extern "C" fn stub_set_achievement(
  _this_ptr: *mut std::ffi::c_void,
  achievement_name: *const std::ffi::c_char,
) -> bool {
  if !achievement_name.is_null() {
    let c_str = unsafe { std::ffi::CStr::from_ptr(achievement_name) };
    println!("Blocked Steam achievement: {}", c_str.to_string_lossy());
  }
  // Return true so the game thinks the achievement unlocked successfully
  true
}
unsafe extern "C" fn stub_store_stats(this_ptr: *mut std::ffi::c_void) -> bool {
  println!("Blocked Steam StoreStats call.");
  // Return true so the game believes the save/store operation succeeded
  true
}

pub fn disable_steam_achievements() {
  let _ = SteamUserStatsHook.init_with_self(move || {
    let stats_instance = SteamUserStatsHook.call();

    if !stats_instance.is_null() {
      unsafe {
        // the first 4 bytes of a C++ class instance point to its vtable
        let vtable_ptr = *stats_instance as *mut *mut std::ffi::c_void;

        // index 7 for SetAchievement (0x1c), index 10 for StoreStats (0x28)
        let achievement_slot = vtable_ptr.add(7) as *mut u8;
        let store_stats_slot = vtable_ptr.add(10) as *mut u8;

        let achievement_bytes =
          (stub_set_achievement as *mut std::ffi::c_void as usize).to_ne_bytes();
        let store_bytes = (stub_store_stats as *mut std::ffi::c_void as usize).to_ne_bytes();

        // patch our stub functions in
        patch_memory(achievement_slot, &achievement_bytes);
        patch_memory(store_stats_slot, &store_bytes);

        println!("Successfully stubbed SteamUserStats vtable offset 0x1c (SetAchievement)");
      }
    }

    stats_instance
  });

  let _ = unsafe { SteamUserStatsHook.enable() };
}
