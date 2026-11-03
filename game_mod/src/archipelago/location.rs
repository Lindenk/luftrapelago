use archipelago_rs::{AsLocationId, Location};
use num_enum::TryFromPrimitive;

use crate::{
  archipelago::items::BASE_ID,
  game_data::plane::{BodyType, EngineType, Plane, WeaponType},
};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum APLocation {
  RauserLevel(RauserLevel),
  Completion(Plane),
  GenericCompletion(u8),
}

impl APLocation {
  pub fn as_rauser_level(self) -> Option<RauserLevel> {
    match self {
      Self::RauserLevel(l) => Some(l),
      _ => None,
    }
  }
}

impl AsLocationId for APLocation {
  fn as_location_id(&self) -> i64 {
    match self {
      // checks start at level 2
      APLocation::RauserLevel(l) => l.as_location_id(),
      APLocation::Completion(p) => {
        // locations should in order of:
        let n = p.weapon as i64 + 5 * p.body as i64 + 5 * 5 * p.engine as i64;
        BASE_ID + 0x600 + n
      }
      APLocation::GenericCompletion(n) => BASE_ID + 0x400 + (*n - 1) as i64,
    }
  }
}

impl TryFrom<i64> for APLocation {
  type Error = ();
  fn try_from(id: i64) -> Result<Self, Self::Error> {
    let id = BASE_ID + id;
    Ok(match id {
      l @ 0x300..0x308 => Self::RauserLevel(RauserLevel(l as u16 - 0x300 + 2)),
      l @ 0x400..0x47e => Self::GenericCompletion(l as u8 + 1),
      l @ 0x600..0x67e => {
        let l = l - 0x600;
        if l == 125 {
          return Ok(Self::Completion(Plane {
            weapon: WeaponType::URauser,
            body: BodyType::URauser,
            engine: EngineType::URauser,
          }));
        }

        // these can't fail for any part < 5, or l < 125
        let w = WeaponType::try_from_primitive((l % 5) as u8).unwrap();
        let b = BodyType::try_from_primitive(((l / 5) % 5) as u8).unwrap();
        let e = EngineType::try_from_primitive((l / 25) as u8).unwrap();
        Self::Completion(Plane {
          weapon: w,
          body: b,
          engine: e,
        })
      }
      _ => return Err(()),
    })
  }
}

impl TryFrom<Location> for APLocation {
  type Error = ();
  fn try_from(value: Location) -> Result<Self, Self::Error> {
    Self::try_from(value.as_location_id())
  }
}

impl From<RauserLevel> for APLocation {
  fn from(value: RauserLevel) -> Self {
    Self::RauserLevel(value)
  }
}

impl From<Plane> for APLocation {
  fn from(value: Plane) -> Self {
    Self::Completion(value)
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct RauserLevel(u16);

impl RauserLevel {
  pub fn try_new(level: u16) -> Option<Self> {
    if level >= 2 && level <= 10 {
      Some(Self(level))
    } else {
      None
    }
  }
  pub fn get(self) -> u16 {
    self.0
  }
}

impl AsLocationId for RauserLevel {
  fn as_location_id(&self) -> i64 {
    BASE_ID + 0x300 + self.0 as i64 - 2
  }
}
