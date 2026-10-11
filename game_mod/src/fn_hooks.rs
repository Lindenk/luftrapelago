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
  pub static LoadSave: extern "C" fn() -> i32 => 0x7f150;
  pub static SaveGame: extern "C" fn(i32) -> i32 => 0x7fe10;
  pub static InitGame: extern "thiscall" fn(*mut std::ffi::c_void, i32) -> () => 0xff2e0;
}

define_external_detour_hooks! {
  pub static Fiopen from "MSVCP100.dll" => "?_Fiopen@std@@YAPAU_iobuf@@PBDHH@Z" : extern "C" fn(
    *const std::ffi::c_char, // filename
    i32,          // std::ios_base::openmode flags
    i32           // file protection/sharing mode
  ) -> *mut std::ffi::c_void;
  pub static OStreamWrite from "MSVCP100.dll" => "?write@?$basic_ostream@DU?$char_traits@D@std@@@std@@QAEAAV12@PBD_J@Z" : extern "C" fn(
    *mut std::ffi::c_void, // this
    *const u8, // data to write
    i64 // number of bytes to write
  ) -> *mut std::ffi::c_void;
}
