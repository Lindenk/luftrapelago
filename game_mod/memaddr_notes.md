BEST SCORE  0x128
BEST COMBO  0x130
GAMES       0x138
TOTAL TIME  0x148

[0x1ab34fa]:
+0x34 = Unlocked items (padding, engine, body, weapon)

+0x20 = games played
+0x24 = total score
+0x32 = player level?
+0x4c = last game time
+0x50 = total time

+0x14 = current score?
+0x44 = last score?
+0x48 = best combo?

+0x58 = fighters killed last
+0x5c = boats killed last
+0x64 = jets killed last

+0x8c = fighters killed total
+0x90 = boats killed total
+0x98 = jets killed total

+0xc8 = missions start

0x12c = Wep Original mission 20 fighters complete (0/1 bool)
0x130 = Wep Original mission 20 fighters progress





Mission layout (40 bytes):
+0x00 = Component (padding, mission num, part num, 0=wep,1=body,2=engine,)
+0x04 = Objective
  0 = Kill X
  1 = Play X games
  2 = Get X score
  3 = Kill X enemy types
  4 = Get xX combo
  5 = Bomb X (With body 4)
  6 = Encounter X
  7 = Kill 5 in a row
+0x08 = Condition
  0 = Total kills
  1 = Kills in one combo
  2 = Falling nonstop
  3 = While undamaged
  4 = Without ever firing
  5 = Firing non-stop
  6 = Without missing
  7 = While on fire
  8 = At MAX combo
  9 = Boosting nonstop
  a = In a set time (? unsure how to set time)
  b = After death
  c = In one game
  d = With your eyes closed
+0x0c = Number required
+0x10 = Enemy type (0 = any, 1 = fighter, etc. See elsewhere)
+0x14 = Is completed (0 = no, 1 = yes and move on to next mission)
+0x18 = Progress
+0x1c = ???
+0x20 = ???
+0x24 = ???



Fighter
Boat
Battleship
Jet
Ace
?
Sub
?


missions 1-3 unlocked at start. 4 requires blimp kill.
