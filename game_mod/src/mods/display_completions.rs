use std::ffi::{c_int, c_void};

use archipelago_rs::AsLocationId;
use num_enum::TryFromPrimitive;

use crate::{
  archipelago::location::APLocation,
  define_detour_hooks,
  fn_hook::get_base_address,
  game_data::{
    game_string::GameString,
    mission::{MissionCode, MissionDescriptor},
    plane::{BodyType, EngineType, Part, Plane, WeaponType},
  },
  patcher::PatchBuilder,
  state::GameState,
};

define_detour_hooks! {
  //static SetPlaneName: extern "thiscall" fn(*mut c_void, c_int, c_int, c_int) -> *mut u8 =>  0x7eeb0;
}

pub fn display_completions() {
  // the call is wonky, just hook the end
  let addr = (get_base_address() + 0x7ef94) as *mut u8;
  PatchBuilder::with_start_addr(addr)
    .use_trampoline()
    .push_all()
    .append(&[
      // set up args
      0x89, 0xc1, // mov ecx, eax, this is the returned string
      0x8b, 0x55, 0xec, // mov edx,[ebp - 0x14], this is the mission code
    ])
    .call(append_completion_status as extern "fastcall" fn(*mut u8, u32) as u32)
    .pop_all()
    .append(&[0x8b, 0xe5, 0x5d]) // recreate the instructions we're overwriting
    .append(&[0xc2, 0x0c, 0x00]) // ret (and clear the 3 original args)
    .write();
}

extern "fastcall" fn append_completion_status(str_ptr: *mut u8, parts: u32) {
  let plane = {
    let wep = ((parts >> 16) & 0xff) as u8;
    let body = (parts & 0xff) as u8;
    let engine = ((parts >> 8) & 0xff) as u8;
    Plane {
      weapon: WeaponType::try_from_primitive(wep).unwrap(),
      body: BodyType::try_from_primitive(body).unwrap(),
      engine: EngineType::try_from_primitive(engine).unwrap(),
    }
  };

  // skip urauser
  if plane.weapon == WeaponType::URauser
    || plane.body == BodyType::URauser
    || plane.engine == EngineType::URauser
  {
    return;
  }

  GameState::with(|client, _, _| {
    let mut s = GameString::copy_from_game(str_ptr);

    let is_completed = client
      .client()
      .checked_locations()
      .map(|l| l.as_location_id())
      .find(|l| *l == APLocation::Completion(plane).as_location_id())
      .is_some();
    let remaining_completions_needed = client
      .client()
      .unchecked_locations()
      .filter_map(|l| APLocation::try_from(l).ok())
      .filter(|l| matches!(l, APLocation::GenericCompletion(_)))
      .count();
    let is_expected = client
      .client()
      .unchecked_locations()
      .map(|l| l.as_location_id())
      .find(|l| *l == APLocation::Completion(plane).as_location_id())
      .is_some();
    let options = client.client().slot_data().clone();

    // this one is done
    if is_completed || (remaining_completions_needed == 0 && options.send_generic_completions) {
      s = s.append(" (Completed)".to_string());
    }
    // handle the case where we only have a set specific number to do
    else if options.send_generic_completions || is_expected {
      s = s.append(" (Needed)".to_string())
    }

    s.assign(str_ptr);
  });
}
