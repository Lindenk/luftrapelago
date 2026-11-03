use std::ops::Not;

use archipelago_rs::AsLocationId;

use crate::{
  define_detour_hooks,
  fn_hook::get_base_address,
  game_data::{
    game_string::GameString,
    mission::{MissionCode, MissionDescriptor},
  },
  patcher::{CodePatch, patch_memory},
  state::GameState,
};

define_detour_hooks! {
  static GenerateMissionString: extern "stdcall" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut u8 => 0xaf950;
}

pub fn draw_mission_hints() {
  // nop out the bit that draws the skulls on missions
  {
    let addr = (get_base_address() + 0xb2047) as *mut u8;
    let end_addr = (get_base_address() + 0xb20e1) as *mut u8;
    unsafe { CodePatch::at(addr).continuation(end_addr).write() };
  }

  // nop out the checks for needing blimp kills, smft mode, etc
  // mission text
  unsafe {
    let addr = (get_base_address() + 0xaf9a4) as *mut u8;
    patch_memory(addr, &[0xeb]); // skip the blimp check
  }
  unsafe {
    let addr = (get_base_address() + 0xaf9de) as *mut u8;
    patch_memory(addr, &[0xeb]); // skip the sfmt check
  }
  unsafe {
    let addr = (get_base_address() + 0xafa0c) as *mut u8;
    patch_memory(addr, &[0xeb]); // skip the mission 5 check
  }

  // sfmt and blimp kill mission check
  unsafe {
    let addr = (get_base_address() + 0xaf917) as *mut u8;
    patch_memory(addr, &[0xb0, 0x00, 0xc3]); // force return false
  }
  // mission text highlighting
  unsafe {
    let addr = (get_base_address() + 0xb169a) as *mut u8;
    patch_memory(addr, &[0xe9, 0x70, 0x01, 0x00, 0x00]); // skip the sfmt/blimp checks
  }

  // append our hints to the mission text (when it gets generated)
  let _ = GenerateMissionStringHook.init_with_self(|this, that| {
    let s = GenerateMissionStringHook.call(this, that);

    // figure out what mission location this is
    let mission_code = MissionCode(unsafe { *(this as *const u32) });
    let mission = MissionDescriptor::try_from(mission_code);

    // get our hints
    enum Hint {
      Hint(String),
      Completed,
      Pending,
      Locked,
      Invalid,
    }
    let hint = if let Ok(mission) = mission {
      GameState::with(|client, _, _| {
        if client.get_parts().find(|p| *p == mission.part).is_none() {
          return Hint::Locked;
        }
        if mission.num >= 5 {
          return Hint::Completed;
        }

        let player = client.client().this_player().clone();
        client
          .client()
          .scouted_locations()
          .find(|l| l.location().same_location(mission.as_location_id()))
          .map(|l| {
            if *l.receiver() == player {
              Hint::Hint(l.item().name().to_string().to_lowercase())
            } else {
              Hint::Hint(format!(
                "{}'s {}",
                l.receiver().name().to_lowercase(),
                l.item().name().to_lowercase()
              ))
            }
          })
          .unwrap_or(Hint::Pending)
      })
      .unwrap_or(Hint::Pending)
    } else {
      Hint::Invalid
    };

    let hint = match hint {
      Hint::Hint(h) => h,
      Hint::Completed => "All checks found".to_string(),
      Hint::Pending => "fetching hint...".to_string(),
      Hint::Invalid => {
        format!("invalid mission! {:x}", mission_code)
      }
      Hint::Locked => {
        GameString::assign_from_str(s, "part locked");
        return s;
      }
    };

    let gs = GameString::copy_from_game(s);
    let gs = gs.append(format!("\n  {}", hint));
    gs.assign(s);

    s
  });
  let _ = unsafe { GenerateMissionStringHook.enable() };
}

// TODO: Hook this in somewhere sane, like at the end of a round
pub fn check_hints() {
  GameState::with(|client, memory, _| {
    let unlocked_parts: Vec<_> = client.get_parts().collect();
    let active_missions = memory.get_all_active_missions(&unlocked_parts);
    //println!("Active missions: {:?}", active_missions);
    let scouted_locations: Vec<_> = client
      .client()
      .scouted_locations()
      .map(|l| l.location().as_location_id())
      .collect();
    //println!("Scouted locations: {:?}", scouted_locations);
    let unhinted_missions: Vec<_> = active_missions
      .iter()
      .filter_map(|(_, m)| {
        let id = m.as_location_id();
        scouted_locations.contains(&id).not().then_some(id)
      })
      .collect();
    //println!("Unhinted missions: {:?}", unhinted_missions);
    if unhinted_missions.len() > 0 {
      client
        .client()
        .scout_locations(unhinted_missions, archipelago_rs::CreateAsHint::New);
    }
  });
}
