#![allow(dead_code)]

use windows::Win32::System::Memory::{
  MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualAlloc,
  VirtualProtect,
};

// Write a patch into game memory
pub unsafe fn patch_memory(address: *mut u8, bytes: &[u8]) {
  let size = bytes.len();
  if size == 0 {
    return;
  }

  let mut old = PAGE_PROTECTION_FLAGS(0);
  unsafe {
    if VirtualProtect(address as *const _, size, PAGE_EXECUTE_READWRITE, &mut old).is_err() {
      panic!("Failed to change memory protection at {:p}", address);
    }
    for (i, &b) in bytes.iter().enumerate() {
      std::ptr::write_volatile(address.add(i), b);
    }
    let _ = VirtualProtect(address as *const _, size, old, &mut old);
  }
}

// Make putting together a code patch easier
#[derive(Debug, Clone, Default)]
pub struct PatchBuilder {
  start: *mut u8,
  patch: Vec<u8>,
  use_trampoline: bool,
}

impl PatchBuilder {
  pub fn with_start_addr(addr: *mut u8) -> Self {
    Self {
      start: addr,
      ..Default::default()
    }
  }

  pub fn append(mut self, bytes: &[u8]) -> Self {
    self.patch.extend(bytes);
    self
  }

  pub fn write(self) {
    unsafe {
      if self.use_trampoline {
        // allocate a block of executable memory
        let ptr = VirtualAlloc(
          None,
          self.patch.len(),
          MEM_COMMIT | MEM_RESERVE,
          PAGE_EXECUTE_READWRITE,
        ) as *mut u8;
        std::ptr::copy_nonoverlapping(self.patch.as_ptr(), ptr, self.patch.len());
        Self::with_start_addr(self.start).jump_to(ptr).write();
      } else {
        patch_memory(self.start, &self.patch);
      };
    }
  }

  pub fn into_bytes(self) -> Vec<u8> {
    self.patch
  }

  // instead of directly writing this into the target address,
  // create a block of new memory holding the patch and jump into
  // it. NOTE: This will not automatically return back to the patch
  // position so be sure to jump to where you need to afterwards.
  // actually make sure you _ret_ back or something because jump offsets
  // will break. Meh. I don't think I care enough to fix this for something
  // I'm using like once
  pub fn use_trampoline(mut self) -> Self {
    self.use_trampoline = true;
    self
  }

  // some convience builder fns
  pub fn jump_to(mut self, addr: *const u8) -> Self {
    // assume we're using the short jump first
    let mut offset = addr as i32 - self.start as i32 - self.patch.len() as i32 - 2;
    if offset >= -128 && offset <= 127 {
      // use a near jump if we can
      self.patch.extend([0xeb, offset as u8]);
    } else {
      offset -= 3; // compensate for the difference in instruction sizes
      self.patch.push(0xe9);
      self.patch.extend(offset.to_le_bytes());
    }
    self
  }
  pub fn call(mut self, addr: u32) -> Self {
    self.patch.push(0xb8); // mov eax (the pointer below to call)
    self.patch.extend(addr.to_le_bytes());
    self.patch.extend([0xff, 0xd0]); // call eax
    self
  }

  pub fn push_all(mut self) -> Self {
    self.patch.extend(&[
      0x9c, 0x60, // pushfd, pushad
    ]);
    self
  }
  pub fn pop_all(mut self) -> Self {
    self.patch.extend(&[
      0x61, 0x9d, // popad, popfd
    ]);
    self
  }
  // test al and jz to the given address (popping all regs/flags)
  pub fn pop_jz(mut self, addr: *const u8) -> Self {
    // do an inverse check then jmp so we can run our pops

    self.patch.extend(&[
      0x85, 0xc0, // test al, al
    ]);
    // assume we're using the short jump first
    let mut offset = addr as i32 - self.start as i32 - self.patch.len() as i32 - 6;
    if offset >= -128 && offset <= 127 {
      // use a short jump if we can
      #[rustfmt::skip]
      self.patch.extend([
        0x75, 0x04, // skip the jump if not zero
        0x61, 0x9d, // pop
        0xeb, offset as u8
      ]);
    } else {
      offset -= 3; // we have 3 more bytes to skip with a long jump
      self.patch.extend([
        0x75, 0x07, // skip the jump if not zero
        0x61, 0x9d, // pop
        0xe9, // set up the jump
      ]);
      self.patch.extend(offset.to_le_bytes());
    }

    self
  }
  // test al and jnz to the given address (popping all regs/flags)
  pub fn pop_jnz(mut self, addr: *const u8) -> Self {
    // do an inverse check then jmp so we can run our pops

    self.patch.extend(&[
      0x85, 0xc0, // test al, al
    ]);
    // assume we're using the short jump first
    let mut offset = addr as i32 - self.start as i32 - self.patch.len() as i32 - 6;
    if offset >= -128 && offset <= 127 {
      // use a short jump if we can
      #[rustfmt::skip]
      self.patch.extend([
        0x74, 0x04, // skip the jump if zero
        0x61, 0x9d, // pop
        0xeb, offset as u8
      ]);
    } else {
      offset -= 3; // we have 3 more bytes to skip with a long jump
      self.patch.extend([
        0x74, 0x07, // skip the jump if zero
        0x61, 0x9d, // pop
        0xe9, // set up the jump
      ]);
      self.patch.extend(offset.to_le_bytes());
    }

    self
  }
}
