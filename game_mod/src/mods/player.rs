use std::ffi::c_void;

use crate::{define_detour_hooks, mods::check_completion, state::GameState};

define_detour_hooks! {
  static PlayerUpdate: extern "thiscall" fn(*mut c_void) -> () => 0xd05d0;
}

pub fn apply_player_mods() {
  let _ = PlayerUpdateHook.init_with_self(|player| {
    let player = player as *mut f32;
    unsafe {
      let hp_ptr = player.byte_add(0x9c + 0x3c);
      //println!("HP: {:?}", hp_ptr.read_volatile());

      GameState::with(|_, _, temp_state| {
        if temp_state.patchup_active {
          hp_ptr.write_volatile(temp_state.current_max_hp);
          temp_state.patchup_active = false;
        }
        if temp_state.deathlink_triggered {
          hp_ptr.write_volatile(0.);
          // handle clearing deathlink in the die hook so we can prevent
          // triggering deathlink when receiving one
        }
      });
    }

    check_completion();

    PlayerUpdateHook.call(player as *mut c_void);
  });
  let _ = unsafe { PlayerUpdateHook.enable() };
}
