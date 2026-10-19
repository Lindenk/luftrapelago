use windows::Win32::System::Memory::{
  PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};

/// Safely patches a region of memory at the given address with a slice of bytes.
/// Handles changing protection to PAGE_EXECUTE_READWRITE and restoring it afterward.
pub unsafe fn patch_memory(address: *mut u8, bytes: &[u8]) -> bool {
  let size = bytes.len();
  if size == 0 {
    return true;
  }

  let mut old_protect = PAGE_PROTECTION_FLAGS(0);

  // 1. Change memory protection to allow writing
  unsafe {
    if VirtualProtect(
      address as *const _,
      size,
      PAGE_EXECUTE_READWRITE,
      &mut old_protect,
    )
    .is_err()
    {
      println!("Failed to change memory protection at {:p}", address);
      return false;
    }
  }

  // 2. Write bytes volatile to prevent compiler optimizations
  for (i, &byte) in bytes.iter().enumerate() {
    unsafe {
      std::ptr::write_volatile(address.add(i), byte);
    }
  }

  // 3. Restore original memory protection
  let _ = unsafe {
    let _ = VirtualProtect(address as *const _, size, old_protect, &mut old_protect);
  };

  true
}

/// Patches a function call into the given memory (preserving all registers).
/// The target must be at least 11 bytes of memory that can be overwritten.
pub unsafe fn patch_fn_call(address: *mut u8, func: extern "C" fn()) {
  let func_addr = func as usize as u32;

  // Construct the assembly stub bytes:
  // 1. 0x60       - PUSHAD (Save all general-purpose registers: EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI)
  // 2. 0x9C       - PUSHFD (Save CPU condition flags)
  // 3. 0xB8 [...] - MOV EAX, func_addr (Load the absolute Rust function address into EAX)
  // 4. 0xFF 0xD0  - CALL EAX (Execute the Rust function)
  // 5. 0x9D       - POPFD (Restore CPU condition flags)
  // 6. 0x61       - POPAD (Restore all general-purpose registers)

  let mut patch_bytes = vec![
    0x60, // PUSHAD
    0x9C, // PUSHFD
    0xB8, // MOV EAX, imm32
  ];

  // Append the 4-byte function address (little-endian)
  patch_bytes.extend_from_slice(&func_addr.to_le_bytes());

  // Append the call and register/flag restoration opcodes
  patch_bytes.extend_from_slice(&[
    0xFF, 0xD0, // CALL EAX
    0x9D, // POPFD
    0x61, // POPAD
  ]);

  // Apply the patch using the memory protection helper
  let _ = unsafe { patch_memory(address, &patch_bytes) };
}

/// Patches a function call using `patch_fn_call`, followed by an unconditional
/// relative jump to a continuation address. Total patch size is 16 bytes.
pub unsafe fn patch_fn_call_with_jump(
  address: *mut u8,
  func: extern "C" fn(),
  continuation_address: *mut u8,
) -> bool {
  unsafe {
    // write the 11-byte call stub
    patch_fn_call(address, func);

    // place a jmp after the stub
    let jmp_address = address.add(11);

    // Calculate the relative offset for the E9 near jump.
    // A 32-bit relative jump (E9) is 5 bytes long, so the CPU calculates the offset
    // from the address immediately following the jump instruction (jmp_address + 5).
    let jmp_instruction_end = jmp_address as usize + 5;
    let relative_offset = (continuation_address as usize - jmp_instruction_end) as i32;

    // Construct the 5-byte JMP instruction: [0xE9, <4-byte relative offset>]
    let mut jmp_bytes = Vec::with_capacity(5);
    jmp_bytes.push(0xE9);
    jmp_bytes.extend_from_slice(&relative_offset.to_le_bytes());

    // Apply the jump bytes right after the call stub
    patch_memory(jmp_address, &jmp_bytes)
  }
}
