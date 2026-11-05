# Luftrausers Archipelago Randomizer

A multiworld randomizer for the game [Luftrausers](https://en.wikipedia.org/wiki/Luftrausers).

## Wait so it randomizes an arcade game?
Yes! And luckily there are enough missions and unlockable parts for a coherent experience.

Other players can find your plane parts and various filler (and traps). Gaining rauser levels, completing missions, and getting configurable plane completions will let you send other players items. 

# Full docs
Refer to [the apworld doc for the game](ap_world/docs/en_Luftrausers.md)

## Quick Setup

1. Grab the APWorld and game mod from the latest release.
2. Install the APWorld wherever the world will be generated
3. Generate the world and host it
4. Copy the game mod `version.dll` to your installation of Luftrausers.
5. **If on Linux** Make sure you install the Windows version of the game and run it through Proton/Wine. This mod was entirely developed on Linux but it was much easier to support cross platform by targeting the Windows version
6. Launch the game. A console window will appear prompting you to enter the AP server's info
7. Get checks and buy bk

# Future Goals

Features planned but not currently implemented include:

- Randomized missions
- More interesting traps (such as palette swap traps)
- Progressive parts with unique upgrades (instead of the current powerup system)
- An in-game UI for connecting to the APWorld (instead of a janky console window)
- Energy link, where firing uses up energy from the pool?

# Known Issues

- Gun jam traps and bullet upgrades are a little jank and don't work all that well with most weapons. The plan is to replace these with progressive parts

# AI Disclaimer
AI was used in a limited capacity during development of this mod:
- There is no AI code in the APWorld, and very minor AI code in the game mod (specifically the `fn_hooks` rust macro, because writing Rust macros by hand is a bad time). Any AI code was heavily audited
- All other code released was written by me
- AI was used to aid in making sense of the decompilation. There are only so many lines of `puVar27` I can sift through before my eyes glaze over
