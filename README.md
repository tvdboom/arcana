<div align="center">

# Arcana
### A Rust-powered auto-battler game with solo and multiplayer modes

<br><br>
[![Play](https://gist.githubusercontent.com/cxmeel/0dbc95191f239b631c3874f4ccf114e2/raw/play.svg)](https://tvdboom.itch.io/tinywar)
<br><br>
</div>

<img src="https://github.com/tvdboom/tinywar/blob/master/assets/images/bg/scenery1.png?raw=true" alt="scenery1">
<img src="https://github.com/tvdboom/tinywar/blob/master/assets/images/bg/scenery2.png?raw=true" alt="scenery2">
<img src="https://github.com/tvdboom/tinywar/blob/master/assets/images/bg/scenery3.png?raw=true" alt="scenery3">
<img src="https://github.com/tvdboom/tinywar/blob/master/assets/images/bg/scenery4.png?raw=true" alt="scenery4">

<br>

## 📜 Overview

TinyWar is a fast-paced, real-time, auto-battler game, where players fight each other 
on a small map, with the sole goal of destroying the enemy base. A never-ending horde 
of units spawn from each base and walk down one of the three lanes. Players can queue 
up units to spawn, decide which lane(s) the units should take, select from a variety 
of boosts to help their units win the battle, and decide on an engagement strategy.
Remember! If your base is destroyed, you lose the game.

## ⚔️ Combat

Units automatically attack enemy units that are in range. A unit can only attack 
one other unit at the same time, and won't change targets until the enemy has died 
or walked out of range. The damage dealt on the enemy is applied after the end of 
the attack animation (every `attack speed` seconds). Note that this means that some 
units apply damage more frequently than others, as the `attack speed` differs per unit.

Every unit has the following combat stats:

- **Health:** How much damage the unit can take before dying.
- **Physical Damage:** Base physical damage dealt on hit.
- **Magic Damage:** Base magical damage dealt on hit.
- **Armor:** Reduces incoming physical damage.
- **Magic Resist:** Reduces incoming magical damage.
- **Armor Penetration:** Reduces the target’s effective armor.
- **Magic Penetration:** Reduces the target’s effective magic resistance.

The damage calculation happens as follows:

1. Calculate defense stats of the defender:  
   `Defender_Armor = max(0, Defender::Armor - Attacker::Armor_Penetration)`  
   `Defender_Magic_Resist = max(0, Defender.Magic_Resist - Attacker.Magic_Penetration)`

2. Calculate damage mitigation using the effective defense stats:  
   `Physical_Damage = Attacker::Physical_Damage * (10 / (10 + Defender_Armor))`  
   `Magic_Damage = Attacker::Magic_Damage * (10 / (10 + Defender_Magic_Resist))`

3. Calculate final damage applied on the defender (always minimum 5):  
   `Total_Damage = max(5, Physical_Damage + Magic_Damage)`

4. Lastly, subtract the damage from the defender's health:  
   `Defender::Health -= Total_Damage`

## ➡️ Lanes

The map consists of three lanes (top/mid/bot) over which units can reach the enemy
base. Players can select which lanes the spawning units will take. The current
selection is displayed on the top left of the screen with arrows. Click on the image
or use the arrow keys to change the selection.

## ⚡ Boosts

Boosts are power-ups that players can use during the game to enhance their units.
Every 30 seconds, a player can choose from 3 boosts. Selected boosts become available
for activation at the top of the screen. Click on a boost to activate it.

Boosts come in two flavors:

- Instant: Apply their effect immediately upon activation. They can be recognized
  by the fact that they don't have any timer indication.
- Timed: Apply their effect for a limited duration. The timer indication on the
  bottom-right of the image indicates its length.

A player can have a maximum of 4 boosts selected/activated at the same time. If a 
player already has 4 boosts when the selection phase starts, they lose the chance to
select a new one. You can only see the enemy's active boosts.

## ♟️ Strategies

You can choose from 4 strategies that specify the rules of engagement for your units.
After selecting a strategy, you must wait 5 seconds before being able to select another
one. The current strategy (and that from the enemy) is shown in the top banner.

- **Attack** (default): Advance until an enemy is in range, then attack.
- **Guard**: Units that are being attacked go into guard stand (only those that can).
  Guarding units have their armor and magic resist increased by 50%, but don't attack.
- **March**: Increase all unit's movement speed by 50% and ignore the enemies. March
  towards the enemy base!
- **Berserk**: Units gain 30% increased attack speed but reduces their armor and magic
  resist by 50%.

## ⌨️ Key bindings

- `escape`: Enter/exit the in-game menu.
- `w-a-s-d`: Move the map.
- `scroll`: Zoom in/out the map.
- `space`: Pause/unpause the game.
- `ctrl + left/right arrow`: Increase/decrease the game's speed (only if host).
- `H`: Toggle the unit information panel.
- `Q`: Toggle the audio settings.

- Use the arrows to select which [lanes](#lanes) spawning units should take.
- Every unit has a key binding to add it to the queue.
