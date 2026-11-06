use std::ffi::c_void;

use archipelago_rs::ClientStatus;

use crate::{
  archipelago::slot::Goal,
  define_detour_hooks,
  game_data::plane::{BodyType, EngineType, Plane, WeaponType},
  mods::{KeybindActions, check_completion, is_pressed},
  state::GameState,
};

define_detour_hooks! {
  static PlayerUpdate: extern "thiscall" fn(*mut c_void) -> () => 0xd05d0;
}

pub fn apply_player_mods() {
  let _ = PlayerUpdateHook.init_with_self(|player| {
    let player = player as *mut f32;
    unsafe {
      let hp_ptr = player.byte_add(0x9c + 0x3c);
      //println!("HP: {:?}", hp_ptr.read_volatile());

      GameState::with(|client, memory, temp_state| {
        if temp_state.patchup_active {
          hp_ptr.write_volatile(temp_state.current_max_hp);
          temp_state.patchup_active = false;
        }
        if temp_state.deathlink_triggered {
          hp_ptr.write_volatile(0.);
          // handle clearing deathlink in the die hook so we can prevent
          // triggering deathlink when receiving one
        }
        if is_pressed(KeybindActions::EndRun) {
          hp_ptr.write_volatile(0.); // end the run
        };

        // check if we launched the URauser if it's our goal
        if Goal::Urauser == client.client().slot_data().goal
          && matches!(
            memory.get_equipped_loadout(),
            Some(Plane {
              weapon: WeaponType::URauser,
              body: BodyType::URauser,
              engine: EngineType::URauser
            })
          )
        {
          let _ = client.client().set_status(ClientStatus::Goal);
        }
      });
    }

    check_completion();

    PlayerUpdateHook.call(player as *mut c_void);
  });
  let _ = unsafe { PlayerUpdateHook.enable() };
}
