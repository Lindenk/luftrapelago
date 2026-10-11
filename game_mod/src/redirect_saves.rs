use std::ffi::CString;

use windows::core::PCSTR;

use crate::fn_hooks::{FiopenHook, OStreamWriteHook};

pub unsafe fn redirect_saves() {
  //let _ = unsafe {
  //  LoadSaveHook.initialize(fn_load_save, || {
  //    // skip the function, ie. don't let the game load save data because it
  //    // should read it's state from the archipelago server
  //    7 // this code means no save data was found
  //  })
  //};
  //let _ = unsafe { LoadSaveHook.enable() };

  //let _ = SaveGameHook.init_with_self(|_| {
  //  println!(
  //    "Loaded save game function at address: 0x{:X}",
  //    SaveGameHook::get_fn_address()
  //  );
  //  // skip the function, ie. don't let the game load save data because it
  //  // should read it's state from the archipelago server
  //  3 // this code means game saved properly
  //});
  //
  unsafe {
    let _ = FiopenHook.init_with_self(|filename, mode, prot| {
      let path = std::ffi::CStr::from_ptr(filename).to_string_lossy();
      if path.contains("save") {
        println!("Save data access, {}", path);

        //let redirected_path = CString::;
        FiopenHook.call(filename, mode, prot)
      } else {
        FiopenHook.call(filename, mode, prot)
      }
    });
    let _ = FiopenHook.enable();

    let _ = OStreamWriteHook.init_with_self(|this, s, n| {
      // block writes to save file
      println!("Writing to stream...");
      OStreamWriteHook.call(this, s, n)
    });
    //let _ = OStreamWriteHook.enable();
  };

  //let _ = unsafe { SaveGameHook.enable() };
}
