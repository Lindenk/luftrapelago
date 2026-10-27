//! C++ std string bindings (just the ones we need)

use crate::define_detour_hooks;

define_detour_hooks! {
  pub static GameStringAssign: extern "thiscall" fn(*mut u8, *const u8, u32) -> *mut u8 => 0xc270;
}

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

  pub fn to_string(&self) -> String {
    // assume we aren't malformed and _backing is in fact a string
    // (if we needed the heap)
    let n = self.size as usize;
    if let Some(v) = &self._backing {
      String::from_utf8_lossy(&v[..n]).into_owned()
    } else {
      String::from_utf8_lossy(&self.data[..n]).into_owned()
    }
  }

  pub fn copy_from_game(s: *const u8) -> Self {
    let bytes = unsafe {
      let size = *(s.add(0x10) as *const u32) as usize;
      let cap = *(s.add(0x14) as *const u32);
      let data: *const u8 = if cap >= 16 {
        *(s as *const *const u8) // heap
      } else {
        s // not-heap
      };
      std::slice::from_raw_parts(data, size)
    };

    // while we could use the info we just got to do this,
    // I'd rather just reuse what we have even if it's slightly
    // more inefficient
    let s = String::from_utf8_lossy(bytes);
    Self::from_str(&s)
  }

  // append to self. consuming because I don't want to do it manually
  pub fn append(self, other: String) -> Self {
    let mut s = self.to_string();
    s.push_str(&other);
    Self::from_str(&s)
  }

  // assign to a game string
  pub fn assign(&self, other: *mut u8) {
    let s = self.to_string();
    GameStringAssignHook::get_fn()(other, s.as_ptr(), s.len() as u32);
  }
}
