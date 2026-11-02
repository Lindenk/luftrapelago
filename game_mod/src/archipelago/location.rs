use archipelago_rs::{AsLocationId, Location};

use crate::archipelago::items::BASE_ID;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum APLocation {
  // Rauser levels
  RauserLevel(RauserLevel),
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
    }
  }
}

impl TryFrom<i64> for APLocation {
  type Error = ();
  fn try_from(id: i64) -> Result<Self, Self::Error> {
    let id = BASE_ID + id;
    Ok(match id {
      l @ 0x300..0x308 => Self::RauserLevel(RauserLevel(l as u16 - 0x300 + 2)),
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
