//! In-place code patching for 32-bit x86.
//!
//! `CodePatch` assembles a small stub at a patch site that can:
//!   - run raw asm with the game's live registers (preamble),
//!   - call a typed `extern "C"` payload with register/immediate args,
//!   - branch on the payload's return value,
//!   - run raw asm after register restore (postamble),
//!   - jump to a continuation address.
//!
//! All pieces are optional. With no payload, `write()` emits only the
//! preamble/postamble and the continuation jump (a "skip this block" patch).

use windows::Win32::System::Memory::{
  PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};

/// Patch a region of memory, flipping protection to RWX and restoring it.
pub unsafe fn patch_memory(address: *mut u8, bytes: &[u8]) -> bool {
  let size = bytes.len();
  if size == 0 {
    return true;
  }

  let mut old = PAGE_PROTECTION_FLAGS(0);
  unsafe {
    if VirtualProtect(address as *const _, size, PAGE_EXECUTE_READWRITE, &mut old).is_err() {
      println!("Failed to change memory protection at {:p}", address);
      return false;
    }
    for (i, &b) in bytes.iter().enumerate() {
      std::ptr::write_volatile(address.add(i), b);
    }
    let _ = VirtualProtect(address as *const _, size, old, &mut old);
  }
  true
}

// ============================================================================
// Registers / args
// ============================================================================

#[derive(Clone, Copy)]
pub enum Reg {
  Eax = 0,
  Ecx = 1,
  Edx = 2,
  Ebx = 3,
  Esp = 4,
  Ebp = 5,
  Esi = 6,
  Edi = 7,
}

#[derive(Clone, Copy)]
enum Arg {
  Reg(Reg),
  Imm(u32),
}

#[derive(Clone, Copy)]
enum Ret {
  Ignore,
  /// payload returns != 0 -> jmp target; == 0 -> fall through
  BranchNonZero(usize),
  /// payload returns == 0 -> jmp target; != 0 -> fall through
  BranchZero(usize),
}

// ============================================================================
// Typed payloads
// ============================================================================

/// Marker: a 4-byte, register-passable C argument on 32-bit x86.
/// Covers integers and any raw pointer (all ABI-identical here).
pub trait Abi4 {}
impl Abi4 for u32 {}
impl Abi4 for i32 {}
impl Abi4 for usize {}
impl<T> Abi4 for *mut T {}
impl<T> Abi4 for *const T {}

/// Return types the stub can handle.
pub trait PayloadRet {
  /// Whether EAX holds a meaningful value to branch on.
  const BRANCHABLE: bool;
}
impl PayloadRet for () {
  const BRANCHABLE: bool = false;
}
impl PayloadRet for u32 {
  const BRANCHABLE: bool = true;
}
impl PayloadRet for i32 {
  const BRANCHABLE: bool = true;
}

/// A function usable as a patch payload. Carries arity + branchability so the
/// builder can validate attached args and branch requests.
pub trait Payload {
  const ARGS: usize;
  const BRANCHABLE: bool;
  fn addr(&self) -> usize;
}

macro_rules! impl_payload {
  ($n:expr; $($t:ident),*) => {
    impl<$($t: Abi4,)* R: PayloadRet> Payload for extern "C" fn($($t),*) -> R {
      const ARGS: usize = $n;
      const BRANCHABLE: bool = R::BRANCHABLE;
      fn addr(&self) -> usize { *self as usize }
    }
  };
}
impl_payload!(0; );
impl_payload!(1; A);
impl_payload!(2; A, B);
impl_payload!(3; A, B, C);
impl_payload!(4; A, B, C, D);

// ============================================================================
// Builder
// ============================================================================

/// Assembles an in-place call/skip stub at a patch site.
///
/// Emitted layout (every bracketed piece is optional):
/// ```text
///   [preamble]                        ; raw asm, LIVE registers
///   [pushfd; pushad]                  ; register preservation
///   [push arg_n .. push arg_0]        ; cdecl order
///   [mov eax, payload; call eax]
///   [add esp, n*4]                    ; cdecl caller cleanup
///   [mov [slot], eax]                 ; stash return for branching
///   [popad; popfd]
///   [postamble]                       ; raw asm, restored registers
///   [cmp [slot],0; jcc target]        ; branch on return
///   [jmp continuation]
/// ```
pub struct CodePatch {
  site: *mut u8,
  payload: Option<usize>,
  expected_args: usize,
  branchable: bool,
  args: Vec<Arg>,
  preserve: bool,
  ret: Ret,
  continuation: Option<usize>,
  preamble: Vec<u8>,
  postamble: Vec<u8>,
}

impl CodePatch {
  /// Begin a patch at `site`. Without a payload or continuation, `write()`
  /// is effectively a no-op (and warns in debug builds).
  pub fn at(site: *mut u8) -> Self {
    Self {
      site,
      payload: None,
      expected_args: 0,
      branchable: false,
      args: Vec::new(),
      preserve: true,
      ret: Ret::Ignore,
      continuation: None,
      preamble: Vec::new(),
      postamble: Vec::new(),
    }
  }

  /// Attach a typed `extern "C"` payload. Records its arity + branchability.
  ///
  /// Note: pass a fn *pointer*, e.g. `foo as extern "C" fn(*mut u8)`, not a
  /// bare fn item.
  pub fn payload<P: Payload>(mut self, f: P) -> Self {
    self.payload = Some(f.addr());
    self.expected_args = P::ARGS;
    self.branchable = P::BRANCHABLE;
    self
  }

  /// Pass a live register value as the next C argument.
  pub fn arg_reg(mut self, r: Reg) -> Self {
    self.args.push(Arg::Reg(r));
    self
  }

  /// Pass a constant as the next C argument.
  pub fn arg_imm(mut self, v: u32) -> Self {
    self.args.push(Arg::Imm(v));
    self
  }

  /// Save/restore all GP regs + flags around the call (default: true).
  /// Disable only if the payload is intended to alter game registers.
  pub fn preserve_registers(mut self, yes: bool) -> Self {
    self.preserve = yes;
    self
  }

  /// If payload returns non-zero, jump to `target` instead of the continuation.
  pub fn branch_if_nonzero(mut self, target: usize) -> Self {
    assert!(self.payload.is_some(), "branch requires a payload");
    assert!(self.branchable, "payload returns () - nothing to branch on");
    self.ret = Ret::BranchNonZero(target);
    self
  }

  /// If payload returns zero, jump to `target` instead of the continuation.
  pub fn branch_if_zero(mut self, target: usize) -> Self {
    assert!(self.payload.is_some(), "branch requires a payload");
    assert!(self.branchable, "payload returns () - nothing to branch on");
    self.ret = Ret::BranchZero(target);
    self
  }

  /// Where to jump when the stub finishes (and the fall-through of a branch).
  pub fn continuation(mut self, addr: *mut u8) -> Self {
    self.continuation = Some(addr as usize);
    self
  }

  /// Raw machine code emitted at the TOP, before register save. Sees live
  /// registers; changes persist into the original code. Must be
  /// position-independent (no relative jmp/call) and stack-balanced.
  pub fn preamble(mut self, bytes: impl Into<Vec<u8>>) -> Self {
    self.preamble = bytes.into();
    self
  }

  /// Raw machine code emitted AFTER register restore, before the exit jump.
  /// Same position-independence + balance rules as `preamble`.
  pub fn postamble(mut self, bytes: impl Into<Vec<u8>>) -> Self {
    self.postamble = bytes.into();
    self
  }

  /// Assemble and write the stub. Returns false if the memory write fails.
  pub unsafe fn write(self) -> bool {
    let base = self.site as usize;
    let mut c: Vec<u8> = Vec::new();

    // 1. preamble (live registers)
    c.extend_from_slice(&self.preamble);

    if let Some(payload) = self.payload {
      assert_eq!(
        self.args.len(),
        self.expected_args,
        "payload takes {} args but {} were attached",
        self.expected_args,
        self.args.len()
      );

      if self.preserve {
        c.push(0x9C); // PUSHFD
        c.push(0x60); // PUSHAD
      }

      // cdecl: first arg at lowest address -> push in reverse
      for a in self.args.iter().rev() {
        match *a {
          Arg::Reg(r) => c.push(0x50 + r as u8), // PUSH r32
          Arg::Imm(v) => {
            c.push(0x68); // PUSH imm32
            c.extend(v.to_le_bytes());
          }
        }
      }

      c.push(0xB8); // MOV EAX, imm32
      c.extend((payload as u32).to_le_bytes());
      c.extend_from_slice(&[0xFF, 0xD0]); // CALL EAX

      let argbytes = (self.args.len() * 4) as u32; // cdecl caller cleanup
      if argbytes != 0 {
        if argbytes <= 127 {
          c.extend_from_slice(&[0x83, 0xC4, argbytes as u8]); // ADD ESP, imm8
        } else {
          c.push(0x81); // ADD ESP, imm32
          c.push(0xC4);
          c.extend(argbytes.to_le_bytes());
        }
      }

      // Stash EAX before popad clobbers it (per-patch leaked, stable address).
      let slot: *mut u32 = match self.ret {
        Ret::Ignore => std::ptr::null_mut(),
        _ => Box::leak(Box::new(0u32)) as *mut u32,
      };
      if !slot.is_null() {
        c.push(0xA3); // MOV [imm32], EAX
        c.extend((slot as u32).to_le_bytes());
      }

      if self.preserve {
        c.push(0x61); // POPAD
        c.push(0x9D); // POPFD
      }

      // 2. postamble (restored registers)
      c.extend_from_slice(&self.postamble);

      // 3. branch on stashed return
      if let Ret::BranchNonZero(t) | Ret::BranchZero(t) = self.ret {
        c.extend_from_slice(&[0x83, 0x3D]); // CMP dword [imm32], imm8
        c.extend((slot as u32).to_le_bytes());
        c.push(0x00);
        let (a, b) = match self.ret {
          Ret::BranchNonZero(_) => (0x0F, 0x85), // JNE rel32
          Ret::BranchZero(_) => (0x0F, 0x84),    // JE  rel32
          _ => unreachable!(),
        };
        c.push(a);
        c.push(b);
        let end = base + c.len() + 4;
        c.extend(((t as i64 - end as i64) as i32).to_le_bytes());
      }
    } else {
      debug_assert!(self.args.is_empty(), "args attached but no payload set");
      debug_assert!(matches!(self.ret, Ret::Ignore), "branch set but no payload");
      // postamble still valid with no payload (runs after preamble)
      c.extend_from_slice(&self.postamble);
    }

    // 4. continuation jump
    if let Some(cont) = self.continuation {
      c.push(0xE9); // JMP rel32
      let end = base + c.len() + 4;
      c.extend(((cont as i64 - end as i64) as i32).to_le_bytes());

      let gap = cont.saturating_sub(base);
      assert!(
        c.len() <= gap,
        "stub is {} bytes but only {} bytes to continuation",
        c.len(),
        gap
      );
    } else {
      debug_assert!(
        self.payload.is_some() || !self.preamble.is_empty() || !self.postamble.is_empty(),
        "patch has neither payload nor continuation - no-op"
      );
    }

    patch_memory(self.site, &c)
  }
}

// ============================================================================
// Small asm helpers for pre/postamble bytes
// ============================================================================

/// Position-independent opcode builders for the common one/two-byte forms.
/// All use absolute addressing so they're safe inside a relocated stub.
pub mod asm {
  use super::Reg;

  /// `mov [dst], r32`  (store a register into an absolute address)
  pub fn store_reg(dst: *const u32, r: Reg) -> Vec<u8> {
    // 89 /r with ModRM = 00 reg 101 (disp32)
    let modrm = 0x05 | ((r as u8) << 3);
    let mut v = vec![0x89, modrm];
    v.extend((dst as u32).to_le_bytes());
    v
  }

  /// `mov r32, [src]`  (load an absolute address into a register)
  pub fn load_reg(r: Reg, src: *const u32) -> Vec<u8> {
    // 8B /r with ModRM = 00 reg 101 (disp32)
    let modrm = 0x05 | ((r as u8) << 3);
    let mut v = vec![0x8B, modrm];
    v.extend((src as u32).to_le_bytes());
    v
  }

  /// `mov r32, imm32`
  pub fn mov_imm(r: Reg, imm: u32) -> Vec<u8> {
    let mut v = vec![0xB8 + r as u8];
    v.extend(imm.to_le_bytes());
    v
  }

  /// `xor r32, r32`  (zero a register)
  pub fn zero_reg(r: Reg) -> Vec<u8> {
    let modrm = 0xC0 | ((r as u8) << 3) | (r as u8);
    vec![0x31, modrm]
  }

  /// `push r32`
  pub fn push_reg(r: Reg) -> Vec<u8> {
    vec![0x50 + r as u8]
  }

  /// `pop r32`
  pub fn pop_reg(r: Reg) -> Vec<u8> {
    vec![0x58 + r as u8]
  }

  /// Concatenate several byte fragments into one buffer.
  pub fn seq<I: IntoIterator<Item = Vec<u8>>>(parts: I) -> Vec<u8> {
    parts.into_iter().flatten().collect()
  }
}
