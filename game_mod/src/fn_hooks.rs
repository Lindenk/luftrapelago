#![allow(unused)]

use std::sync::OnceLock;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;

static BASE_ADDRESS: OnceLock<usize> = OnceLock::new();

pub fn get_base_address() -> &'static usize {
  BASE_ADDRESS.get_or_init(|| unsafe { GetModuleHandleA(None).unwrap().0 as usize })
}

// Macro to automate hooking to local functions
#[macro_export]
macro_rules! define_detour_hooks {
  (
    $(
      $vis:vis static $name:ident : extern $abi:literal fn ($($arg:ty),*) -> $ret:ty => $offset:expr;
    )*
  ) => {
    paste::paste! {
      $(
        // Recreate the Fn type
        $vis type [<$name Fn>] = extern $abi fn ($($arg),*) -> $ret;

        // Create the hook
        detour::static_detour! {
          $vis static [<$name Hook>]: extern $abi fn ($($arg),*) -> $ret;
        }

        // Provide repetative helper fns
        impl [<$name Hook>] {
          pub fn get_fn_address() -> usize {
            get_base_address() + $offset
          }
          pub fn get_fn() -> [<$name Fn>] {
            let target_fn_addr = Self::get_fn_address();
            unsafe { std::mem::transmute(target_fn_addr) }
          }

          pub fn init_with_self<F>(self, f: F) -> detour::Result<Self>
          where
            F: Fn($($arg),*) -> $ret + Send + Sync + 'static,
          {
            unsafe { self.initialize(Self::get_fn(), f) }
          }
        }
      )*
    }
  };
}

// Macro to automate hooking to dlls
#[macro_export]
macro_rules! define_external_detour_hooks {
  (
    $(
      $vis:vis static $name:ident from $lib:literal => $sym:literal : extern $abi:literal fn ($($arg:ty),*) -> $ret:ty;
    )*
  ) => {
    paste::paste! {
      $(
        $vis type [<$name Fn>] = extern $abi fn ($($arg),*) -> $ret;

        detour::static_detour! {
          $vis static [<$name Hook>]: extern $abi fn ($($arg),*) -> $ret;
        }

        impl [<$name Hook>] {
          pub fn get_fn_address() -> usize {
            unsafe {
              use windows::Win32::System::LibraryLoader::{LoadLibraryA, GetProcAddress};
              let lib_handle = LoadLibraryA(windows::core::s!($lib))
                .expect(concat!("Failed to load library: ", $lib));
              let proc_addr = GetProcAddress(lib_handle, windows::core::s!($sym))
                .expect(concat!("Failed to find symbol: ", $sym));
              proc_addr as usize
            }
          }
          pub fn get_fn() -> [<$name Fn>] {
            let target_fn_addr = Self::get_fn_address();
            unsafe { std::mem::transmute(target_fn_addr) }
          }

          pub fn init_with_self<F>(self, f: F) -> detour::Result<Self>
          where
            F: Fn($($arg),*) -> $ret + Send + Sync + 'static,
          {
            unsafe { self.initialize(Self::get_fn(), f) }
          }
        }
      )*
    }
  };
}

define_detour_hooks! {
  pub static Entry: extern "C" fn() -> () => 0x1dc925;
  pub static MasterGameInit: extern "stdcall" fn(*mut std::ffi::c_void) -> () => 0x9d8b0;
  pub static ResultsScreen: extern "fastcall" fn(*mut std::ffi::c_void) -> () => 0xeb4f0;
  pub static DisplayUnlocks: extern "fastcall" fn(*mut std::ffi::c_void) -> () => 0xb5310;
}

define_external_detour_hooks! {
  pub static SteamUserStats from "steam_api.dll" => "SteamUserStats" : extern "C" fn() -> *mut *mut std::ffi::c_void;
}
