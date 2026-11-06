from collections import OrderedDict


# wow strenums do not do what you want most of the time
class NameBaseMeta(type):
    def __iter__(cls):
        for key, value in vars(cls).items():
            if not key.startswith("_") and isinstance(value, str):
                yield value


class NameBase(metaclass=NameBaseMeta):
    pass


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
    PowerupHealth = "Max Health Up"
    PowerupFireRate = "Fire Rate Up"
    PowerupBonusBullets = "Bullets Up"


class Filler(NameBase):
    Patchup = "Patch Up"
    Blank = "Screen Clear"
    TrapJam = "Gun Jam"
    TrapStall = "Engine Stall"


#   TrapSpawn = "Wave Spawn"
#    TrapMissiles = "Anti-air Missiles"


class MiscItems(NameBase):
    ProgressiveRank = "Rank Key"
    Victory = "Victory"
    CompletionToken = "Completion"


# Locations

# there's only 6 missions per part in the base game,
# but add some padding if we want to randomize the
# number of missions later
weapon_missions: OrderedDict[tuple[str, int], str] = OrderedDict(
    ((part, mission_no), f"Mission - Weapon {part} {mission_no}") for part in WeaponNames for mission_no in range(1, 6)
)
body_missions: OrderedDict[tuple[str, int], str] = OrderedDict(
    ((part, mission_no), f"Mission - Body {part} {mission_no}") for part in BodyNames for mission_no in range(1, 6)
)
engine_missions: OrderedDict[tuple[str, int], str] = OrderedDict(
    ((part, mission_no), f"Mission - Engine {part} {mission_no}") for part in EngineNames for mission_no in range(1, 6)
)

level_names = [f"Rauser Level {i}" for i in range(2, 11)]


class VictoryLocations(NameBase):
    LaunchURauser = "Launch URauser"
    FinishedUniqueCompletions = "Finished Unique Completions"
    ScoreAttack = "Score Attack"


# 126 potential unique game 'completions'
generic_completions = [f"{n} Completion(s)" for n in range(1, 126)]


# the key is the name, the value is a dict for how to build it
_unique_planes: "OrderedDict[str, dict[str, int]]" = OrderedDict(
    {
        "ORIGINAL": {"weapon": 0, "body": 0, "engine": 0},
        "BEAMRAUSER": {"weapon": 1, "body": 0, "engine": 0},
        "SPREADRAUSER": {"weapon": 2, "body": 0, "engine": 0},
        "MAVERICK": {"weapon": 3, "body": 0, "engine": 0},
        "HUBBLE": {"weapon": 4, "body": 0, "engine": 0},
        "HEAVY": {"weapon": 0, "body": 1, "engine": 0},
        "TEDDY": {"weapon": 1, "body": 1, "engine": 0},
        "KOZILEK": {"weapon": 2, "body": 1, "engine": 0},
        "SKULLRAUSER": {"weapon": 3, "body": 1, "engine": 0},
        "TENTONNER": {"weapon": 4, "body": 1, "engine": 0},
        "STALKER": {"weapon": 0, "body": 2, "engine": 0},
        "JOUSTER": {"weapon": 1, "body": 2, "engine": 0},
        "THE DOG": {"weapon": 2, "body": 2, "engine": 0},
        "LUFTPIRATE": {"weapon": 3, "body": 2, "engine": 0},
        "BULL": {"weapon": 4, "body": 2, "engine": 0},
        "DUMBO": {"weapon": 0, "body": 3, "engine": 0},
        "INSPECTOR": {"weapon": 1, "body": 3, "engine": 0},
        "BOOM": {"weapon": 2, "body": 3, "engine": 0},
        "VENGEANCE": {"weapon": 3, "body": 3, "engine": 0},
        "PINOCHIO": {"weapon": 4, "body": 3, "engine": 0},
        "LUFTBOMBER": {"weapon": 0, "body": 4, "engine": 0},
        "THE PIDGEON": {"weapon": 1, "body": 4, "engine": 0},
        "THE EAGLE": {"weapon": 2, "body": 4, "engine": 0},
        "PASSENGER": {"weapon": 3, "body": 4, "engine": 0},
        "DROP IT": {"weapon": 4, "body": 4, "engine": 0},
        "JETRAUSER": {"weapon": 0, "body": 0, "engine": 1},
        "LASER KNIGHT": {"weapon": 1, "body": 0, "engine": 1},
        "GARDNER": {"weapon": 2, "body": 0, "engine": 1},
        "HUNTER": {"weapon": 3, "body": 0, "engine": 1},
        "MECHRAUSER": {"weapon": 4, "body": 0, "engine": 1},
        "G.N.": {"weapon": 0, "body": 1, "engine": 1},
        "TUBERAUSER": {"weapon": 1, "body": 1, "engine": 1},
        "THE PIG": {"weapon": 2, "body": 1, "engine": 1},
        "GRIZZLY HUGS": {"weapon": 3, "body": 1, "engine": 1},
        "SMALLS": {"weapon": 4, "body": 1, "engine": 1},
        "RAMJET": {"weapon": 0, "body": 2, "engine": 1},
        "CONTACT BEAM": {"weapon": 1, "body": 2, "engine": 1},
        "KARATERAUSER": {"weapon": 2, "body": 2, "engine": 1},
        "HELLRAUSER SUPERBOOST": {"weapon": 3, "body": 2, "engine": 1},
        "STAFF PICK": {"weapon": 4, "body": 2, "engine": 1},
        "KAMIKAZE": {"weapon": 0, "body": 3, "engine": 1},
        "BATMAN": {"weapon": 1, "body": 3, "engine": 1},
        "LOCKON": {"weapon": 2, "body": 3, "engine": 1},
        "DANGER ZONE": {"weapon": 3, "body": 3, "engine": 1},
        "THE FURY": {"weapon": 4, "body": 3, "engine": 1},
        "THE SITUATION": {"weapon": 0, "body": 4, "engine": 1},
        "STORMRAUSER": {"weapon": 1, "body": 4, "engine": 1},
        "AIR PATROL": {"weapon": 2, "body": 4, "engine": 1},
        "TRAILBLAZER": {"weapon": 3, "body": 4, "engine": 1},
        "STRIKERAUSER": {"weapon": 4, "body": 4, "engine": 1},
        "MOSQUITO": {"weapon": 0, "body": 0, "engine": 2},
        "PILOT PARTY": {"weapon": 1, "body": 0, "engine": 2},
        "MARAUDER": {"weapon": 2, "body": 0, "engine": 2},
        "HUMMINGBIRD": {"weapon": 3, "body": 0, "engine": 2},
        "CORPSEMAKER": {"weapon": 4, "body": 0, "engine": 2},
        "THE SLUG": {"weapon": 0, "body": 1, "engine": 2},
        "GROUND CONTROL": {"weapon": 1, "body": 1, "engine": 2},
        "GREEDHEAD": {"weapon": 2, "body": 1, "engine": 2},
        "SPOOK": {"weapon": 3, "body": 1, "engine": 2},
        "SCIENTIST": {"weapon": 4, "body": 1, "engine": 2},
        "TANDEM": {"weapon": 0, "body": 2, "engine": 2},
        "TEETHRAUSER": {"weapon": 1, "body": 2, "engine": 2},
        "BLADERAUSER": {"weapon": 2, "body": 2, "engine": 2},
        "RELICRAUSER": {"weapon": 3, "body": 2, "engine": 2},
        "INFILTRATOR": {"weapon": 4, "body": 2, "engine": 2},
        "FOOD POISONING": {"weapon": 0, "body": 3, "engine": 2},
        "RESUARTFUL": {"weapon": 1, "body": 3, "engine": 2},
        "THE VULTURE": {"weapon": 2, "body": 3, "engine": 2},
        "DEVESTATOR": {"weapon": 3, "body": 3, "engine": 2},
        "UVB-76": {"weapon": 4, "body": 3, "engine": 2},
        "GUNSHIP": {"weapon": 0, "body": 4, "engine": 2},
        "TORPEDORAUSER": {"weapon": 1, "body": 4, "engine": 2},
        "BOB RAUS": {"weapon": 2, "body": 4, "engine": 2},
        "ASSAULTBIRD": {"weapon": 3, "body": 4, "engine": 2},
        "WIDOWMAKER": {"weapon": 4, "body": 4, "engine": 2},
        "PENGUIN": {"weapon": 0, "body": 0, "engine": 3},
        "NAUTILUS": {"weapon": 1, "body": 0, "engine": 3},
        "SPACE TIME": {"weapon": 2, "body": 0, "engine": 3},
        "BREAKFAST": {"weapon": 3, "body": 0, "engine": 3},
        "TERROR FROM THE deep": {"weapon": 4, "body": 0, "engine": 3},
        "KILOMAN SAN": {"weapon": 0, "body": 1, "engine": 3},
        "HUMPBACK": {"weapon": 1, "body": 1, "engine": 3},
        "THE JELLY": {"weapon": 2, "body": 1, "engine": 3},
        "PIRANHA": {"weapon": 3, "body": 1, "engine": 3},
        "WARTURTLE": {"weapon": 4, "body": 1, "engine": 3},
        "JAWS": {"weapon": 0, "body": 2, "engine": 3},
        "NIGHTMARE": {"weapon": 1, "body": 2, "engine": 3},
        "SHIPWRECK": {"weapon": 2, "body": 2, "engine": 3},
        "HAMMERHEAD": {"weapon": 3, "body": 2, "engine": 3},
        "SINKER": {"weapon": 4, "body": 2, "engine": 3},
        "SEABOMB": {"weapon": 0, "body": 3, "engine": 3},
        "SWORDFISH": {"weapon": 1, "body": 3, "engine": 3},
        "JACQUES": {"weapon": 2, "body": 3, "engine": 3},
        "SAMURAUSER": {"weapon": 3, "body": 3, "engine": 3},
        "GUNFISH": {"weapon": 4, "body": 3, "engine": 3},
        "GARFIELD": {"weapon": 0, "body": 4, "engine": 3},
        "HOLY DIVER": {"weapon": 1, "body": 4, "engine": 3},
        "A-RAUSER": {"weapon": 2, "body": 4, "engine": 3},
        "TRICKSTER": {"weapon": 3, "body": 4, "engine": 3},
        "MOOSE": {"weapon": 4, "body": 4, "engine": 3},
        "FLOATRAUSER": {"weapon": 0, "body": 0, "engine": 4},
        "SPACERAUSER": {"weapon": 1, "body": 0, "engine": 4},
        "MOONRAUSER": {"weapon": 2, "body": 0, "engine": 4},
        "ALUCARD": {"weapon": 3, "body": 0, "engine": 4},
        "ENFORCER": {"weapon": 4, "body": 0, "engine": 4},
        "NIMBUS": {"weapon": 0, "body": 1, "engine": 4},
        "BILLY 40": {"weapon": 1, "body": 1, "engine": 4},
        "FORK": {"weapon": 2, "body": 1, "engine": 4},
        "POSTMAN": {"weapon": 3, "body": 1, "engine": 4},
        "BRUTAL RAUSER": {"weapon": 4, "body": 1, "engine": 4},
        "BONERAUSER": {"weapon": 0, "body": 2, "engine": 4},
        "DRILL KILLER": {"weapon": 1, "body": 2, "engine": 4},
        "GRASSHOPPER": {"weapon": 2, "body": 2, "engine": 4},
        "SWITCHBLADE": {"weapon": 3, "body": 2, "engine": 4},
        "HELLRAUSER HOVER": {"weapon": 4, "body": 2, "engine": 4},
        "MAJOR TOM": {"weapon": 0, "body": 3, "engine": 4},
        "BIOHAZARD MK. I": {"weapon": 1, "body": 3, "engine": 4},
        "HAWKRAUSER": {"weapon": 2, "body": 3, "engine": 4},
        "LOVELESS": {"weapon": 3, "body": 3, "engine": 4},
        "BIOHAZARD MK. II": {"weapon": 4, "body": 3, "engine": 4},
        "FLOATBOMBER": {"weapon": 0, "body": 4, "engine": 4},
        "ORBITAL RAUSER": {"weapon": 1, "body": 4, "engine": 4},
        "HOUSEBOAT": {"weapon": 2, "body": 4, "engine": 4},
        "UNHEALTHY BOEING": {"weapon": 3, "body": 4, "engine": 4},
        "FIST OF GOD": {"weapon": 4, "body": 4, "engine": 4},
    }
)
unique_planes: "OrderedDict[str, dict[str, int]]" = OrderedDict(
    {f"Completion {name}": parts for (name, parts) in _unique_planes.items()}
)
