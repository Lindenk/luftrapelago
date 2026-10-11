from enum import StrEnum

# Items
class WeaponNames(StrEnum):
  OriginalWeapon = "Original (Weapon)"
  Laser = "Laser"
  Spread = "Spread"
  Missles = "Missles"
  Cannon = "Cannon"
  URauserWeapon = "URauser (Weapon)"

class BodyNames(StrEnum):
  OriginalBody = "Original (Body)"
  Armor = "Armor"
  Melee = "Melee"
  Nuke = "Nuke"
  Bomb = "Bomb"
  URauserBody = "URauser (Body)"

class EngineNames(StrEnum):
  OriginalEngine = "Original (Engine)"
  Superboost = "Superboost"
  Gungine = "Gungine"
  Underwater = "Underwater"
  Hover = "Hover"
  URauserEngine = "URauser (Engine)"

class Bonuses(StrEnum):
  PowerupDamage = "Schadensbooster"
  PowerupHealth = "Heilung"
  PowerupFireRate = "Schnellfeuer"

class Filler(StrEnum):
  Patchup = "Patch Up"
  Blank = "Screen Clear"
  TrapJam = "Gun Jam"
  TrapSpawn = "Wave Spawn"
  TrapMissles = "Anti-air Missles"

class MiscItems(StrEnum):
  ProgressiveRank = "Progressive Rank"
  Victory = "Victory"

# Locations

# there's only 6 missions per part in the base game,
# but add some padding if we want to randomize the
# number of missions later
weapon_missions: dict[tuple[str, int], str] = {
  (part, mission_no): f"Mission - Weapon ${part} ${mission_no}"
  for part in WeaponNames
  for mission_no in range(16)
}
body_missions: dict[tuple[str, int], str] = {
  (part, mission_no): f"Mission - Body ${part} ${mission_no}"
  for part in BodyNames
  for mission_no in range(16)
}
engine_missions: dict[tuple[str, int], str] = {
  (part, mission_no): f"Mission - Engine ${part} ${mission_no}"
  for part in EngineNames
  for mission_no in range(16)
}

level_names = [
  f"Rauser Level ${i}"
  for i in range(1, 11)
]

class MiscLocations(StrEnum):
  LaunchURauser = "Launch URauser"
  ScoreAttack = "Score Attack"

# 126 potential unique game 'completions'
generic_completions = [
  f"Unique Completion ${n}" for n in range(1, 126)
]

""" TODO: Specific plane completions
class PlaneNames(StrEnum):
  Plane000 = "original"
  Plane000 = "beamrauser"
  Plane000 = "spreadrauser"
  Plane000 = "maverick"
  Plane000 = "hubble"
  Plane000 = "heavy"
  Plane000 = "teddy"
  Plane000 = "kozilek"
  Plane000 = "skullrauser"
  Plane000 = "tentonner"
  Plane000 = "stalker"
  Plane000 = "jouster"
  Plane000 = "the dog"
  Plane000 = "luftpirate"
  Plane000 = "bull"
  Plane000 = "dumbo"
  Plane000 = "inspector"
  Plane000 = "boom"
  Plane000 = "vengeance"
  Plane000 = "pinochio"
  Plane000 = "luftbomber"
  Plane000 = "the pidgeon"
  Plane000 = "the eagle"
  Plane000 = "passenger"
  Plane000 = "drop it"
  Plane000 = "jetrauser"
  Plane000 = "laser knight"
  Plane000 = "gardner"
  Plane000 = "hunter"
  Plane000 = "mechrauser"
  Plane000 = "g.n."
  Plane000 = "tuberauser"
  Plane000 = "the pig"
  Plane000 = "grizzly hugs"
  Plane000 = "smalls"
  Plane000 = "ramjet"
  Plane000 = "contact beam"
  Plane000 = "karaterauser"
  Plane000 = "hellrauser"
  Plane000 = "staff pick"
  Plane000 = "kamikaze"
  Plane000 = "batman"
  Plane000 = "lockon"
  Plane000 = "danger zone"
  Plane000 = "the fury"
  Plane000 = "the situation"
  Plane000 = "stormrauser"
  Plane000 = "air patrol"
  Plane000 = "trailblazer"
  Plane000 = "strikerauser"
  Plane000 = "mosquito"
  Plane000 = "pilot party"
  Plane000 = "marauder"
  Plane000 = "hummingbird"
  Plane000 = "corpsemaker"
  Plane000 = "the slug"
  Plane000 = "ground control"
  Plane000 = "greedhead"
  Plane000 = "spook"
  Plane000 = "scientist"
  Plane000 = "tandem"
  Plane000 = "teethrauser"
  Plane000 = "bladerauser"
  Plane000 = "relicrauser"
  Plane000 = "infiltrator"
  Plane000 = "food poisoning"
  Plane000 = "resuartful"
  Plane000 = "the vulture"
  Plane000 = "devestator"
  Plane000 = "UVB-76"
  Plane000 = "gunship"
  Plane000 = "torpedorauser"
  Plane000 = "bob raus"
  Plane000 = "assaultbird"
  Plane000 = "widowmaker"
  Plane000 = "penguin"
  Plane000 = "nautilus"
  Plane000 = "space time"
  Plane000 = "breakfast"
  Plane000 = "terror from the deep"
  Plane000 = "kiloman san"
  Plane000 = "humpback"
  Plane000 = "the jelly"
  Plane000 = "piranha"
  Plane000 = "warturtle"
  Plane000 = "jaws"
  Plane000 = "nightmare"
  Plane000 = "shipwreck"
  Plane000 = "hammerhead"
  Plane000 = "sinker"
  Plane000 = "seabomb"
  Plane000 = "swordfish"
  Plane000 = "jacques"
  Plane000 = "samurauser"
  Plane000 = "gunfish"
  Plane000 = "garfield"
  Plane000 = "holy diver"
  Plane000 = "a-rauser"
  Plane000 = "trickster"
  Plane000 = "moose"
  Plane000 = "floatrauser"
  Plane000 = "spacerauser"
  Plane000 = "moonrauser"
  Plane000 = "alucard"
  Plane000 = "enforcer"
  Plane000 = "nimbus"
  Plane000 = "billy 40"
  Plane000 = "fork"
  Plane000 = "postman"
  Plane000 = "brutal rauser"
  Plane000 = "bonerauser"
  Plane000 = "drill killer"
  Plane000 = "grasshopper"
  Plane000 = "switchblade"
  Plane000 = "hellrauser"
  Plane000 = "major tom"
  Plane000 = "biohazard mk. I"
  Plane000 = "hawkrauser"
  Plane000 = "loveless"
  Plane000 = "biohazard mk. II"
  Plane000 = "floatbomber"
  Plane000 = "orbital rauser"
  Plane000 = "houseboat"
  Plane000 = "unhealthy boeing"
  Plane000 = "fist of god"
  Plane000 = "URAUSER"
"""
