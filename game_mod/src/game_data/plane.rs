#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum WeaponType {
  Original = 0,
  Laser = 1,
  Spread = 2,
  Missles = 3,
  Cannon = 4,
  URauser = 5,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum BodyType {
  Original = 0,
  Armor = 1,
  Melee = 2,
  Nuke = 3,
  Bomb = 4,
  URauser = 5,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum EngineType {
  Original = 0,
  Superboost = 1,
  Gungine = 2,
  Underwater = 3,
  Hover = 4,
  URauser = 5,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Plane {
  weapon: WeaponType,
  body: BodyType,
  engine: EngineType,
}
