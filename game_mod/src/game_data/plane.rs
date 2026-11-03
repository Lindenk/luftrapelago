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
/*
impl PlaneConfig {
  pub fn name(&self) -> Option<&str> {
    let code = (self.weapon as u32) << 16 | (self.body as u32) << 8 | (self.engine as u32);
    PARTS_TO_PLANE_NAME.get(&code).map(|v| *v)
  }

  // TODO: Use actual try from I guess, even though I'd rather this be an option?
  pub fn try_from_name(name: &str) -> Option<Self> {
    PLANE_NAME_TO_PARTS
      .get(name)
      .map(|code| {
        let weapon = WeaponType::try_from((code >> 16) as u8);
        let body = BodyType::try_from((code >> 8) as u8);
        let engine = EngineType::try_from(*code as u8);

        match (weapon, body, engine) {
          (Ok(weapon), Ok(body), Ok(engine)) => Some(Self {
            weapon,
            body,
            engine,
          }),
          _ => None,
        }
      })
      .flatten()
  }
}

// Map plane name combos to their parts
// TODO: Load this from json in the apworld dir

const PARTS_TO_PLANE_NAME: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
  PLANE_NAME_TO_PARTS
    .iter()
    .map(|(&name, &value)| (value, name))
    .collect()
});
const PLANE_NAME_TO_PARTS: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
  HashMap::from([
    ("original", 0x00_00_00),
    ("beamrauser", 0x01_00_00),
    ("spreadrauser", 0x02_00_00),
    ("maverick", 0x03_00_00),
    ("hubble", 0x04_00_00),
    ("heavy", 0x00_01_00),
    ("teddy", 0x01_01_00),
    ("kozilek", 0x02_01_00),
    ("skullrauser", 0x03_01_00),
    ("tentonner", 0x04_01_00),
    ("stalker", 0x00_02_00),
    ("jouster", 0x01_02_00),
    ("the dog", 0x02_02_00),
    ("luftpirate", 0x03_02_00),
    ("bull", 0x04_02_00),
    ("dumbo", 0x00_03_00),
    ("inspector", 0x01_03_00),
    ("boom", 0x02_03_00),
    ("vengeance", 0x03_03_00),
    ("pinochio", 0x04_03_00),
    ("luftbomber", 0x00_04_00),
    ("the pidgeon", 0x01_04_00),
    ("the eagle", 0x02_04_00),
    ("passenger", 0x03_04_00),
    ("drop it", 0x04_04_00),
    ("jetrauser", 0x00_00_01),
    ("laser knight", 0x01_00_01),
    ("gardner", 0x02_00_01),
    ("hunter", 0x03_00_01),
    ("mechrauser", 0x04_00_01),
    ("g.n.", 0x00_01_01),
    ("tuberauser", 0x01_01_01),
    ("the pig", 0x02_01_01),
    ("grizzly hugs", 0x03_01_01),
    ("smalls", 0x04_01_01),
    ("ramjet", 0x00_02_01),
    ("contact beam", 0x01_02_01),
    ("karaterauser", 0x02_02_01),
    ("hellrauser_superboost", 0x03_02_01),
    ("staff pick", 0x04_02_01),
    ("kamikaze", 0x00_03_01),
    ("batman", 0x01_03_01),
    ("lockon", 0x02_03_01),
    ("danger zone", 0x03_03_01),
    ("the fury", 0x04_03_01),
    ("the situation", 0x00_04_01),
    ("stormrauser", 0x01_04_01),
    ("air patrol", 0x02_04_01),
    ("trailblazer", 0x03_04_01),
    ("strikerauser", 0x04_04_01),
    ("mosquito", 0x00_00_02),
    ("pilot party", 0x01_00_02),
    ("marauder", 0x02_00_02),
    ("hummingbird", 0x03_00_02),
    ("corpsemaker", 0x04_00_02),
    ("the slug", 0x00_01_02),
    ("ground control", 0x01_01_02),
    ("greedhead", 0x02_01_02),
    ("spook", 0x03_01_02),
    ("scientist", 0x04_01_02),
    ("tandem", 0x00_02_02),
    ("teethrauser", 0x01_02_02),
    ("bladerauser", 0x02_02_02),
    ("relicrauser", 0x03_02_02),
    ("infiltrator", 0x04_02_02),
    ("food poisoning", 0x00_03_02),
    ("resuartful", 0x01_03_02),
    ("the vulture", 0x02_03_02),
    ("devestator", 0x03_03_02),
    ("uvb-76", 0x04_03_02),
    ("gunship", 0x00_04_02),
    ("torpedorauser", 0x01_04_02),
    ("bob raus", 0x02_04_02),
    ("assaultbird", 0x03_04_02),
    ("widowmaker", 0x04_04_02),
    ("penguin", 0x00_00_03),
    ("nautilus", 0x01_00_03),
    ("space time", 0x02_00_03),
    ("breakfast", 0x03_00_03),
    ("terror from the deep", 0x04_00_03),
    ("kiloman san", 0x00_01_03),
    ("humpback", 0x01_01_03),
    ("the jelly", 0x02_01_03),
    ("piranha", 0x03_01_03),
    ("warturtle", 0x04_01_03),
    ("jaws", 0x00_02_03),
    ("nightmare", 0x01_02_03),
    ("shipwreck", 0x02_02_03),
    ("hammerhead", 0x03_02_03),
    ("sinker", 0x04_02_03),
    ("seabomb", 0x00_03_03),
    ("swordfish", 0x01_03_03),
    ("jacques", 0x02_03_03),
    ("samurauser", 0x03_03_03),
    ("gunfish", 0x04_03_03),
    ("garfield", 0x00_04_03),
    ("holy diver", 0x01_04_03),
    ("a-rauser", 0x02_04_03),
    ("trickster", 0x03_04_03),
    ("moose", 0x04_04_03),
    ("floatrauser", 0x00_00_04),
    ("spacerauser", 0x01_00_04),
    ("moonrauser", 0x02_00_04),
    ("alucard", 0x03_00_04),
    ("enforcer", 0x04_00_04),
    ("nimbus", 0x00_01_04),
    ("billy 40", 0x01_01_04),
    ("fork", 0x02_01_04),
    ("postman", 0x03_01_04),
    ("brutal rauser", 0x04_01_04),
    ("bonerauser", 0x00_02_04),
    ("drill killer", 0x01_02_04),
    ("grasshopper", 0x02_02_04),
    ("switchblade", 0x03_02_04),
    ("hellrauser_hover", 0x04_02_04),
    ("major tom", 0x00_03_04),
    ("biohazard mk. i", 0x01_03_04),
    ("hawkrauser", 0x02_03_04),
    ("loveless", 0x03_03_04),
    ("biohazard mk. ii", 0x04_03_04),
    ("floatbomber", 0x00_04_04),
    ("orbital rauser", 0x01_04_04),
    ("houseboat", 0x02_04_04),
    ("unhealthy boeing", 0x03_04_04),
    ("fist of god", 0x04_04_04),
    ("urauser", 0x05_05_05),
  ])
});
*/
