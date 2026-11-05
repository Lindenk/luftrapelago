use std::ffi::{c_int, c_void};

use crate::define_external_detour_hooks;

define_external_detour_hooks! {
  static SDLGetKeyboardState from "SDL2.dll" => "SDL_GetKeyboardState": extern "C" fn (*mut c_int) -> *const u8;
  static SDLGetScancodeFromKey from "SDL2.dll" => "SDL_GetScancodeFromKey": extern "C" fn (i32) -> i32;
  static SDLJoystickOpen from "SDL2.dll" => "SDL_JoystickOpen": extern "C" fn (i32) -> *mut c_void;
  static SDLJoystickGetButton from "SDL2.dll" => "SDL_JoystickGetButton": extern "C" fn (*mut c_void, i32) -> u8;
}

pub enum KeybindActions {
  EndRun,
}

// todo: add debouncing
pub fn is_pressed(action: KeybindActions) -> bool {
  match action {
    // on my steam controller the Y button is 13? that seems wrong but idk I'll
    // other people with different controllers give feedback
    KeybindActions::EndRun => check_keydown('r' as i32) || check_joybutton(13),
  }
}

fn check_keydown(key: i32) -> bool {
  let keyboard_state = SDLGetKeyboardStateHook::get_fn()(std::ptr::null_mut());
  if keyboard_state.is_null() {
    return false;
  }
  let scancode = SDLGetScancodeFromKeyHook::get_fn()(key);
  unsafe { *keyboard_state.add(scancode as usize) != 0 }
}

fn check_joybutton(button: i32) -> bool {
  let joystick = SDLJoystickOpenHook::get_fn()(0); // just assume joystick 1
  if joystick.is_null() {
    return false;
  }

  //for i in 0..16 {
  //  if SDLJoystickGetButtonHook::get_fn()(joystick, i) != 0 {
  //    println!("Currently pressed: Button index {}", i);
  //  }
  //}

  SDLJoystickGetButtonHook::get_fn()(joystick, button) != 0
}
