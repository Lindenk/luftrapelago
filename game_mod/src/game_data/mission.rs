use std::{
  fmt::LowerHex,
  ops::{Div, Rem},
};

use archipelago_rs::{AsLocationId, Location};
use num_enum::TryFromPrimitive;

use crate::{
  archipelago::items::BASE_ID,
  game_data::plane::{BodyType, EngineType, Part, WeaponType},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct MissionCode(pub u32);

impl LowerHex for MissionCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let val = self.0;
    std::fmt::LowerHex::fmt(&val, f)
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MissionDescriptor {
  pub num: u8,
  pub part: Part,
  pub is_completed: bool,
}

impl AsLocationId for MissionDescriptor {
  fn as_location_id(&self) -> i64 {
    let mut id = BASE_ID;
    id += self.num as i64;
    id += match self.part {
      Part::Weapon(w) => (w as i64) * 5,
      Part::Body(b) => (b as i64) * 5 + 0x100,
      Part::Engine(e) => (e as i64) * 5 + 0x200,
    };

    id
  }
}

// apparently you straight up can't implement TryFrom generically at all
impl TryFrom<i64> for MissionDescriptor {
  type Error = ();
  fn try_from(id: i64) -> Result<Self, Self::Error> {
    let id = id - BASE_ID;

    let part_type = id >> 8;
    let part_num = (id & 0xff).div(5) as u8;
    let mission_num = (id & 0xff).rem(5) as u8;

    let part = match part_type {
      0 => Part::Weapon(WeaponType::try_from_primitive(part_num).map_err(|_| ())?),
      1 => Part::Body(BodyType::try_from_primitive(part_num).map_err(|_| ())?),
      2 => Part::Engine(EngineType::try_from_primitive(part_num).map_err(|_| ())?),
      _ => return Err(()),
    };

    Ok(Self {
      part,
      num: mission_num,
      is_completed: false,
    })
  }
}
impl TryFrom<Location> for MissionDescriptor {
  type Error = ();
  fn try_from(value: Location) -> Result<Self, Self::Error> {
    Self::try_from(value.as_location_id())
  }
}

impl TryFrom<MissionCode> for MissionDescriptor {
  type Error = ();
  fn try_from(c: MissionCode) -> Result<Self, Self::Error> {
    let part_type = c.0 & 0xff;
    let part_num = (c.0 >> 8) & 0xff;
    let mission_num = (c.0 >> 16) & 0xff;

    let part = match part_type {
      0 => WeaponType::try_from_primitive(part_num as u8)
        .ok()
        .map(|w| Part::Weapon(w)),
      1 => BodyType::try_from_primitive(part_num as u8)
        .ok()
        .map(|b| Part::Body(b)),
      2 => EngineType::try_from_primitive(part_num as u8)
        .ok()
        .map(|e| Part::Engine(e)),
      _ => None,
    }
    .ok_or(())?;

    Ok(Self {
      part,
      num: mission_num as u8,
      is_completed: false,
    })
  }
}

impl From<MissionDescriptor> for MissionCode {
  fn from(v: MissionDescriptor) -> Self {
    let code = match v.part {
      Part::Weapon(w) => 0 | ((w as u32) << 8),
      Part::Body(b) => 1 | ((b as u32) << 8),
      Part::Engine(e) => 2 | ((e as u32) << 8),
    };

    Self(code | ((v.num as u32) << 16))
  }
}
