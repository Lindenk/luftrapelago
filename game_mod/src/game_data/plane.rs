use num_enum::TryFromPrimitive;

use crate::archipelago::items::APItem;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Part {
  Weapon(WeaponType),
  Body(BodyType),
  Engine(EngineType),
}

impl From<WeaponType> for Part {
  fn from(value: WeaponType) -> Self {
    Self::Weapon(value)
  }
}
impl From<BodyType> for Part {
  fn from(value: BodyType) -> Self {
    Self::Body(value)
  }
}
impl From<EngineType> for Part {
  fn from(value: EngineType) -> Self {
    Self::Engine(value)
  }
}

impl TryFrom<APItem> for Part {
  type Error = ();
  fn try_from(value: APItem) -> Result<Self, Self::Error> {
    match value {
      APItem::Weapon(w) => Ok(Self::Weapon(w)),
      APItem::Body(b) => Ok(Self::Body(b)),
      APItem::Engine(e) => Ok(Self::Engine(e)),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive)]
#[repr(u8)]
pub enum WeaponType {
  Random = 255,
  Original = 0,
  Laser = 1,
  Spread = 2,
  Missles = 3,
  Cannon = 4,
  URauser = 5,
}

impl TryFrom<Part> for WeaponType {
  type Error = ();
  fn try_from(value: Part) -> Result<Self, Self::Error> {
    match value {
      Part::Weapon(w) => Ok(w),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive)]
#[repr(u8)]
pub enum BodyType {
  Random = 255,
  Original = 0,
  Armor = 1,
  Melee = 2,
  Nuke = 3,
  Bomb = 4,
  URauser = 5,
}
impl TryFrom<Part> for BodyType {
  type Error = ();
  fn try_from(value: Part) -> Result<Self, Self::Error> {
    match value {
      Part::Body(b) => Ok(b),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Hash, Eq, Clone, Copy, PartialEq, PartialOrd, Ord, TryFromPrimitive)]
#[repr(u8)]
pub enum EngineType {
  Random = 255,
  Original = 0,
  Superboost = 1,
  Gungine = 2,
  Underwater = 3,
  Hover = 4,
  URauser = 5,
}
impl TryFrom<Part> for EngineType {
  type Error = ();
  fn try_from(value: Part) -> Result<Self, Self::Error> {
    match value {
      Part::Engine(e) => Ok(e),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct Plane {
  pub weapon: WeaponType,
  pub body: BodyType,
  pub engine: EngineType,
}
