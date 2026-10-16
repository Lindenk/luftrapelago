class NameBase:
  @classmethod
  def iter(cls) -> list[str]:
    """Dynamically collects all string constants from the class and its superclasses."""
    return [
      getattr(cls, attr)
      for attr in dir(cls)
      if not attr.startswith("_") and isinstance(getattr(cls, attr), str)
    ]

# Items
class WeaponNames(NameBase):
  OriginalWeapon = "Original (Weapon)"
  Laser = "Laser"
  Spread = "Spread"
  Missles = "Missiles"
  Cannon = "Cannon"
  URauserWeapon = "URauser (Weapon)"

class BodyNames(NameBase):
  OriginalBody = "Original (Body)"
  Armor = "Armor"
  Melee = "Melee"
  Nuke = "Nuke"
  Bomb = "Bomb"
  URauserBody = "URauser (Body)"

class EngineNames(NameBase):
  OriginalEngine = "Original (Engine)"
  Superboost = "Superboost"
  Gungine = "Gungine"
  Underwater = "Underwater"
  Hover = "Hover"
  URauserEngine = "URauser (Engine)"

class Bonuses(NameBase):
  PowerupDamage = "Schadensbooster"
  PowerupHealth = "Heilung"
  PowerupFireRate = "Schnellfeuer"

class Filler(NameBase):
  Patchup = "Patch Up"
  Blank = "Screen Clear"
  TrapJam = "Gun Jam"
  TrapSpawn = "Wave Spawn"
  TrapMissiles = "Anti-air Missiles"

class MiscItems(NameBase):
  ProgressiveRank = "Progressive Rank"
  Victory = "Victory"
  CompletionToken = "Completion Token"

# Locations

# there's only 6 missions per part in the base game,
# but add some padding if we want to randomize the
# number of missions later
weapon_missions: dict[tuple[str, int], str] = {
  (part, mission_no): f"Mission - Weapon {part} {mission_no}"
  for part in WeaponNames.iter()
  for mission_no in range(1, 6)
}
body_missions: dict[tuple[str, int], str] = {
  (part, mission_no): f"Mission - Body {part} {mission_no}"
  for part in BodyNames.iter()
  for mission_no in range(1, 6)
}
engine_missions: dict[tuple[str, int], str] = {
  (part, mission_no): f"Mission - Engine {part} {mission_no}"
  for part in EngineNames.iter()
  for mission_no in range(1, 6)
}

level_names = [
  f"Rauser Level {i}"
  for i in range(1, 11)
]

class VictoryLocations(NameBase):
  LaunchURauser = "Launch URauser"
  FinishedUniqueCompletions = "Finished Unique Completions"
  ScoreAttack = "Score Attack"

# 126 potential unique game 'completions'
generic_completions = [
  f"Unique Completion {n}" for n in range(1, 126)
]

# the key is the name, the value is a dict for how to build it
# loaded from json so the exact same file can be used on the
# mod side
def _load_unique_planes():
  import json
  from pathlib import Path

  current_dir = Path(__file__).resolve().parent
  planes_file = Path.joinpath(current_dir, "planes.json")
  with open(planes_file) as f:
    planes = json.load(f)

  return {f"Completion {name}": build for name, build in planes.items()}

unique_planes: dict[str, dict[str, int]] = _load_unique_planes()
