use crate::game_data::plane::{BodyType, EngineType, Part, WeaponType};

pub const BASE_ID: i64 = 0x1f700000;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum APItem {
  // Parts: Weapons (0x0000 - 0x0005)
  Weapon(WeaponType),
  // Parts: Bodies (0x0020 - 0x0025)
  Body(BodyType),
  // Parts: Engines (0x0040 - 0x0045)
  Engine(EngineType),
  // Powerups (0x0080 - 0x0082)
  PowerupHealth,
  PowerupFireRate,
  PowerupBonusBullets,
  // Misc (0x00C0 - 0x00C1)
  ProgressiveRank,
  CompletionToken,
  // Filler (0x0100 - 0x0101)
  Patchup,
  Blank,
  // Traps (0x0140)
  TrapJam,
  TrapStall,
}

impl APItem {
  pub fn id(&self) -> i64 {
    let id = match self {
      Self::Weapon(w) => *w as i64,
      Self::Body(b) => 0x0020 + (*b as i64),
      Self::Engine(e) => 0x0040 + (*e as i64),
      Self::PowerupHealth => 0x0080,
      Self::PowerupFireRate => 0x0081,
      Self::PowerupBonusBullets => 0x0082,
      Self::ProgressiveRank => 0x00C0,
      Self::CompletionToken => 0x00C1,
      Self::Patchup => 0x0100,
      Self::Blank => 0x0101,
      Self::TrapJam => 0x0140,
      Self::TrapStall => 0x0141,
    };
    id + BASE_ID
  }

  pub fn as_weapon(&self) -> Option<WeaponType> {
    match self {
      Self::Weapon(w) => Some(*w),
      _ => None,
    }
  }
  pub fn as_body(&self) -> Option<BodyType> {
    match self {
      Self::Body(b) => Some(*b),
      _ => None,
    }
  }
  pub fn as_engine(&self) -> Option<EngineType> {
    match self {
      Self::Engine(e) => Some(*e),
      _ => None,
    }
  }

  pub fn as_part(&self) -> Option<Part> {
    match self {
      Self::Weapon(w) => Some(Part::Weapon(*w)),
      Self::Body(b) => Some(Part::Body(*b)),
      Self::Engine(e) => Some(Part::Engine(*e)),
      _ => None,
    }
  }
}

impl TryFrom<i64> for APItem {
  type Error = &'static str;

  fn try_from(value: i64) -> Result<Self, Self::Error> {
    let id = value - BASE_ID;
    match id {
      0x0000..=0x0005 => {
        let wt = WeaponType::try_from(id as u8).map_err(|_| "Invalid Weapon code")?;
        Ok(Self::Weapon(wt))
      }
      0x0020..=0x0025 => {
        let bt = BodyType::try_from((id - 0x0020) as u8).map_err(|_| "Invalid Body code")?;
        Ok(Self::Body(bt))
      }
      0x0040..=0x0045 => {
        let et = EngineType::try_from((id - 0x0040) as u8).map_err(|_| "Invalid Engine code")?;
        Ok(Self::Engine(et))
      }
      0x0080 => Ok(Self::PowerupHealth),
      0x0081 => Ok(Self::PowerupFireRate),
      0x0082 => Ok(Self::PowerupBonusBullets),
      0x00C0 => Ok(Self::ProgressiveRank),
      0x00C1 => Ok(Self::CompletionToken),
      0x0100 => Ok(Self::Patchup),
      0x0101 => Ok(Self::Blank),
      0x0140 => Ok(Self::TrapJam),
      0x0141 => Ok(Self::TrapStall),
      _ => Err("Unknown item code"),
    }
  }
}
