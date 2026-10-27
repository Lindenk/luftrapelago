use std::{
  ffi::c_void,
  sync::atomic::{AtomicI32, Ordering},
};

use crate::{define_detour_hooks, fn_hooks::get_base_address, util::djb2};

define_detour_hooks! {
  static NewTextNode: extern "stdcall" fn(*mut c_void, *mut TextNodeHandle, &TextNodeCoords) -> *mut TextNodeHandle => 0x3e350;
  static SetFont: extern "thiscall" fn(*mut c_void, *const u8, *const u8, *const u8, *const i32) -> () => 0x122a00;
  static SetText: extern "thiscall" fn(*mut c_void, *const GameString) -> () => 0x1227e0;
  static SetPosition: extern "thiscall" fn(*mut c_void, f32, f32) -> () => 0x122e90;
  static SetColor: extern "thiscall" fn(*mut c_void, i32) -> () => 0x122840;
  pub static GameStringAssign: extern "thiscall" fn(*mut c_void, *mut c_void, u32) -> c_void => 0xc270;
}

pub const ASSET_LIBRARY: usize = 0x01ab32f4;

// a c++ std::string (more or less)
#[repr(C)]
pub struct GameString {
  data: [u8; 16],
  size: u32,
  cap: u32,
  _backing: Option<Vec<u8>>,
}
impl GameString {
  pub fn from_str(s: &str) -> Self {
    let bytes = s.as_bytes();
    let mut gs = Self {
      data: [0; 16],
      size: bytes.len() as u32,
      cap: 15,
      _backing: None,
    };

    if bytes.len() <= 15 {
      // store it directly in the object
      gs.data[..bytes.len()].copy_from_slice(bytes);
    } else {
      // store the str on the heap. The game only ever copies it so it's safe to
      // allow a ref back to our "c++ string" for the call
      let mut v = Vec::with_capacity(bytes.len() + 1);
      v.extend_from_slice(bytes);
      v.push(0);

      gs.data[..4].copy_from_slice(&(v.as_ptr() as u32).to_le_bytes());
      gs.cap = v.len() as u32;
      gs._backing = Some(v);
    }

    gs
  }
}

#[repr(C)]
#[derive(Clone)]
pub struct TextNodeHandle {
  pub raw: *mut c_void,  // lr::Text*
  pub ctrl: *mut c_void, // MSVC shared_ptr control block ({ vtbl, uses, weaks })
}

#[repr(C)]
pub struct TextNodeCoords {
  pub x: f32,
  pub y: f32,
}

pub struct TextNode {
  handle: TextNodeHandle,
}

impl TextNode {
  pub fn new() -> Self {
    let mut handle = TextNodeHandle {
      raw: std::ptr::null_mut(),
      ctrl: std::ptr::null_mut(),
    };
    Self::with_handle(&mut handle)
  }
  pub fn from_handle(handle: *mut TextNodeHandle) -> Self {
    Self {
      handle: unsafe { (*handle).clone() }, // TODO: Use the smart pointer
    }
  }
  pub fn with_handle(handle: *mut TextNodeHandle) -> Self {
    let library = unsafe { *(ASSET_LIBRARY as *const *mut c_void) };

    let coords = TextNodeCoords { x: 264., y: 23. };

    NewTextNodeHook::get_fn()(library, handle, &coords);
    let mut node = Self::from_handle(handle);

    // set the font
    unsafe {
      let p = node.raw();

      *(p.add(0x138) as *mut u32) = 0;
      *(p.add(0x13c) as *mut u32) = 0x3f80_0000; // 1.0f

      let shader = GameString::from_str("gfx/shaders/font.shader");
      let fnt = GameString::from_str("gfx/fonts/fnts.fnt");
      let png = GameString::from_str("gfx/fonts/fnts_0.png");
      let hash = djb2("gfx/shaders/font.shader"); // seed 5381, *33 — see note
      node.call_set_font(&shader, &fnt, &png, &(hash as i32));
    }

    node.set_color_index(7);
    node.set_position(0., 0.);

    node
  }
  #[inline]
  pub fn raw(&self) -> *mut u8 {
    self.handle.raw as *mut u8
  }
  // TODO: See if we need to leak these instead of handling them correctly.
  // (it probably doesn't make much of a difference to leak as long
  // as we only init them once)
  pub fn leak(self) -> *mut c_void {
    let raw = self.handle.raw;
    std::mem::forget(self);
    raw
  }

  pub fn tick(&mut self) {
    unsafe {
      let node = self.raw() as *mut c_void;
      let vtbl = *(self.raw() as *const *const usize);
      // index 1 (offset +4) = Text::vfunction2 = update/rebuild
      let update: unsafe extern "thiscall" fn(*mut c_void) = std::mem::transmute(*vtbl.add(1));
      update(node);
    }
  }

  // the string setting function is a wonky calling convention
  unsafe fn call_set_text(&self, src: *const GameString) {
    let target = SetTextHook::get_fn_address();
    let node = self.raw() as *mut c_void;

    unsafe {
      core::arch::asm!(
        "push {node}",
        "call {target}",  // fn does cleans up the param
        node = in(reg) node,
        target = in(reg) target,
        inout("eax") src => _, // source string in EAX
        out("ecx") _,
        out("edx") _,
        // the other regs are preservered by fn
      )
    }
  }

  // font handling is also wonky...
  unsafe fn call_set_font(
    &self,
    //descriptor: *mut c_void,   // the ECX "this" scratch object
    shader: *const GameString, // param_1
    fnt: *const GameString,    // param_2
    png: *const GameString,    // param_3
    hash: *const i32,          // param_4
  ) {
    let node = self.raw() as *mut c_void;
    unsafe {
      // [0]=shader [1]=fnt [2]=png [3]=hash [4]=target
      let args: [usize; 5] = [
        shader as usize,
        fnt as usize,
        png as usize,
        hash as usize,
        SetFontHook::get_fn_address(),
      ];

      core::arch::asm!(
          // stack args, pushed in reverse so param_1 ends up at [esp+4] after the call
          "push dword ptr [{a} + 12]", // hash   -> param_4
          "push dword ptr [{a} + 8]",  // png    -> param_3
          "push dword ptr [{a} + 4]",  // fnt    -> param_2
          "push dword ptr [{a} + 0]",  // shader -> param_1
          "call dword ptr [{a} + 16]", // target (indirect call)
          // If the callee is `ret 0x10` (callee-clean), esp is already balanced.
          // If it's plain `ret` (caller-clean), uncomment:
          // "add esp, 16",
          a = in(reg) args.as_ptr(),   // only ONE scratch reg needed
          in("eax") node,
          in("ecx") node,
          out("edx") _,                // caller-saved, callee may clobber
          // ebx/esi/edi are preserved by the target's prologue
      );
    }
  }

  // text manip
  pub fn set_position(&mut self, x: f32, y: f32) {
    SetPositionHook::get_fn()(self.raw() as *mut c_void, x, y);
  }
  pub fn set_color_index(&mut self, index: i32) {
    SetColorHook::get_fn()(self.raw() as *mut c_void, index);
  }
  pub fn set_text(&mut self, text: &str) {
    //println!("Setting text inside node: {}", text);

    // convert the string to a c++ std::string (more or less)
    let s = GameString::from_str(text);
    unsafe {
      self.call_set_text(&s);
    }
  }
}

unsafe fn atomic_dec(ptr: *mut i32) -> i32 {
  unsafe { (*(ptr as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1 }
}
// control-block vtable entries: [0] = destroy managed object, [1] = free control block
type CtrlVfn = unsafe extern "fastcall" fn(*mut c_void, usize);
impl Drop for TextNode {
  // It looks like we get passed back a smart pointer so decrement our
  // reference to it
  fn drop(&mut self) {
    let ctrl = self.handle.ctrl as *mut i32;
    if ctrl.is_null() {
      return;
    }
    unsafe {
      if atomic_dec(ctrl.add(1)) == 0 {
        // ctrl[0] = vtable
        let vtbl = *(ctrl as *const *const usize);
        let destroy: CtrlVfn = std::mem::transmute(*vtbl.add(0));
        destroy(ctrl as *mut c_void, 0);
        if atomic_dec(ctrl.add(2)) == 0 {
          let free_ctrl: CtrlVfn = std::mem::transmute(*vtbl.add(1));
          free_ctrl(ctrl as *mut c_void, 0);
        }
      }
    }
  }
}
