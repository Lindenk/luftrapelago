//! This is a system to help find locations in memory in a way that
//! doesn't rely on a specific build of the game. The general idea is
//! similar to how cheat engines work, but with a little more sauce

use std::{
  collections::HashMap,
  sync::{LazyLock, Mutex},
};

use crate::fn_hook::get_base_address;

static ADDR_CACHE: LazyLock<Mutex<HashMap<String, usize>>> = LazyLock::new(|| Default::default());

#[derive(Debug, Hash, Clone)]
pub struct MemoryMatcher {
  match_units: Vec<MatchUnit>,
}

#[derive(Debug, Hash, Clone)]
enum MatchUnit {
  Match(Vec<u8>),
  MatchFromCache(String),
  Skip(usize),
  FollowRel,
  ValueHere,
  AddressHere,
  CacheAs(String),
}

#[derive(Debug, Hash, Clone)]
struct MatchResult {
  addr: usize,
  name: String,
}

#[derive(Debug, Hash, Clone)]
pub enum MatchError {
  MultipleFound(u8),
  NotFound,
}

impl MemoryMatcher {
  pub fn new() -> Self {
    Self {
      match_units: vec![],
    }
  }

  // match the given bytes exactly
  pub fn match_bytes(mut self, bytes: &[u8]) -> Self {
    self.match_units.push(MatchUnit::Match(Vec::from(bytes)));
    self
  }
  // skip this many bytes
  pub fn skip(mut self, n: usize) -> Self {
    self.match_units.push(MatchUnit::Skip(n));
    self
  }
  // follow the next 4 bytes as a pointer. If it's valid, start matching
  // at it's destination
  pub fn follow_rel(mut self) -> Self {
    self.match_units.push(MatchUnit::FollowRel);
    self
  }

  // match an address from the cache. It will need to be entered into cache
  // before this matcher is used
  pub fn match_from_cache(mut self, name: String) -> Self {
    self.match_units.push(MatchUnit::MatchFromCache(name));
    self
  }

  // running this match will return the 4 byte value at this spot in the match
  pub fn value_at(mut self) -> Self {
    self.match_units.push(MatchUnit::ValueHere);
    self
  }
  // running this match will return the address at this spot in the match
  pub fn addr_at(mut self) -> Self {
    self.match_units.push(MatchUnit::AddressHere);
    self
  }

  // cache the addr here under the given name, to be used later
  pub fn cache_addr_at(mut self, name: String) -> Self {
    self.match_units.push(MatchUnit::CacheAs(name));
    self
  }

  fn matches_at(&self, mut addr: *const u8) -> Option<MatchResult> {
    let mut target_addr = None;
    let mut cache_name = None;

    for mu in self.match_units.iter() {
      match mu {
        MatchUnit::Match(bytes) => {
          let slice = unsafe { std::slice::from_raw_parts(addr, bytes.len()) };
          if slice != bytes {
            return None;
          }
          addr = unsafe { addr.byte_add(bytes.len()) };
        }
        MatchUnit::MatchFromCache(name) => {
          if let Some(a) = ADDR_CACHE.lock().unwrap().get(name) {
            let slice = unsafe { std::slice::from_raw_parts(addr, size_of::<usize>()) };
            if slice != a.to_le_bytes() {
              return None;
            }
            addr = unsafe { addr.byte_add(size_of::<usize>()) };
          } else {
            println!("WARNING: Address not cached {}", name);
            return None;
          }
        }
        MatchUnit::Skip(n) => addr = unsafe { addr.byte_add(*n) },
        MatchUnit::FollowRel => {
          let new_addr: isize = unsafe { (addr as *const isize).read_unaligned() };
          let new_addr: *const u8 = ((addr as isize) + new_addr + 4) as *const u8;

          if (*get_base_address() as *const u8) <= new_addr
            && new_addr <= (*get_base_address() + 0x30_0000) as *const u8
          {
            addr = new_addr;
          } else {
            return None;
          }
        }

        // keep track of this address to return
        MatchUnit::ValueHere => {
          let v = unsafe { (addr as *const usize).read_unaligned() };
          target_addr = Some(v);
        }
        MatchUnit::AddressHere => {
          target_addr = Some(addr as usize);
        }
        MatchUnit::CacheAs(name) => {
          cache_name = Some(name.clone());
        }
      }
    }

    if let Some(addr) = target_addr
      && let Some(name) = cache_name
    {
      Some(MatchResult { addr, name })
    } else {
      println!(
        "WARNING: Match did not include a target or name! {:?}",
        self
      );
      None
    }
  }

  pub fn find(matchers: &[Self]) -> Vec<MatchError> {
    // just hardcode this for now
    let search_start = *get_base_address();

    let mut matches = vec![];
    for addr in search_start..(search_start + 0x30_0000) {
      for matcher in matchers {
        let addr = addr as *const u8;
        if let Some(a) = matcher.matches_at(addr) {
          matches.push(a);
        }
      }
    }

    // sanity check our results
    matchers.iter().map(|m| m.)

    if matches.len() > 2 {
      return Err(MatchError::MultipleFound(matches.len() as u8));
    }

    if let Some(m) = matches.iter().next() {
      ADDR_CACHE
        .lock()
        .unwrap()
        .entry(m.name.clone())
        .and_modify(|x| println!("WARNING: Attempted to insert {} into cache switch", x))
        .or_insert(m.addr);

      Ok(m.addr as u32)
    } else {
      Err(MatchError::NotFound)
    }
  }
}
