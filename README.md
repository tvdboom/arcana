<div align="center">

# Arcana
### A Rust-powered build-based RPG with tactical PvP combat

<br><br>
[![Play](https://gist.githubusercontent.com/cxmeel/0dbc95191f239b631c3874f4ccf114e2/raw/play.svg)](https://tvdboom.itch.io/arcana)
<br><br>
</div>

<img src="https://github.com/tvdboom/arcana/blob/master/assets/images/bg/scenery1.png?raw=true" alt="scenery1">
<img src="https://github.com/tvdboom/arcana/blob/master/assets/images/bg/scenery2.png?raw=true" alt="scenery2">
<img src="https://github.com/tvdboom/arcana/blob/master/assets/images/bg/scenery3.png?raw=true" alt="scenery3">
<img src="https://github.com/tvdboom/arcana/blob/master/assets/images/bg/scenery4.png?raw=true" alt="scenery4">

<br>

## 📜 Overview

Arcana is an intense, build-based RPG that pits players against one another in high-stakes,
tactical PvP combat. Set in a dark-fantasy world of arcane machinery and shifting realities,
players must master the balance between rigorous planning and real-time execution.

### Core Pillars
 
* Strategic Planning: Manage your Action Points (AP) to train stats, master professions,
  craft gear, and prepare your character for the duels ahead.
* Deep Build Customization: Choose from distinct races, classes, subclasses, pets and transformations.
* Tactical PvP Combat: Engage in fluid, peer-to-peer combat where timing your abilities and
  managing cooldowns can mean the difference between victory and defeat.
* Persistent Progression: Every victory earns stakes that impact your character's journey
  and remain saved to your persistent profile.

Step into the void, forge your legend, and claim your place in the Arcana.

<br>

## ⚔️ Combat Mechanics

Combat in Arcana is a real-time simulation driven by stats, timing, and active effects.
Each fighter attacks automatically at an interval determined by their attack speed, while
active abilities can be cast at the cost of mana and cooldowns.

### 1. Attack Interval and Timing

Every entity's attack timing is dictated by their **Attack Period**, which represents the
duration (in seconds) between basic auto-attacks:

$$\text{Attack Period} = \text{clamp}\left(\frac{2.0}{\text{Effective Attack Speed}}, 0.2, 10.0\right)$$

* **Effective Attack Speed** is modified by active effects:
  * `Freeze`: Multiplies speed by $1.0 + \text{attack\_speed\_pct} / 100.0$ (capped at $0.1$ minimum).
  * `BeastFrenzy`: Multiplies speed by $1.0 + \text{attack\_speed\_pct} / 100.0$.

---

### 2. Basic Attack Resolution Steps

When an attack triggers, it undergoes a sequential resolution process:

#### **Step A: Miss Chance**
An attack may miss entirely if the attacker is afflicted with **Blind**:
$$\text{Miss Chance} = \text{clamp}\left(\sum \frac{\text{Blind miss\_pct}}{100.0}, 0.0, 0.90\right)$$
If a random roll is below the miss chance, the attack fails.

#### **Step B: Dodge Chance**
If the attacker does not miss, and the defender is able to move (not `Immobilized`), the defender has a chance to dodge based on initiative differences:
$$\text{Dodge Chance} = \text{clamp}\left(0.18 + (\text{Defender Initiative} - \text{Attacker Initiative}) \times 0.018, 0.08, 0.70\right)$$

* **Effective Initiative** ($I$) is modified by:
  * `Haste`: Multiplies initiative by $1.0 + \text{initiative\_pct} / 100.0$.
  * `Paranoia`: Multiplies initiative by $(1.0 - \text{initiative\_pct} / 100.0)$ (capped at $0.0$ minimum).

#### **Step C: Critical Strike Roll**
An attack has a chance to land a critical strike (inflicting double damage):
$$\text{Total Crit Chance} = \text{clamp}\left(\text{Base Crit} + \sum \frac{\text{Focus crit\_chance\_pct}}{100.0}, 0.0, 1.0\right)$$

---

### 3. Damage Calculation Formula

If the attack successfully hits, the raw damage is computed as follows:

$$\text{Base Damage} = \frac{\text{Effective Attack}^2}{\max(\text{Effective Attack} + \text{Effective Defense}, 1.0)}$$

$$\text{Final Damage} = \text{Base Damage} \times \text{Variance} \times \text{Incoming Multiplier} \times \text{Bleed Multiplier} \times \text{Crit Multiplier}$$

* **Variance**: A random multiplier between $0.85$ and $1.15$ rolled per hit.
* **Effective Attack**:
  * `Berserk`, `Empower`, and `BeastFrenzy` each apply $(1.0 + \text{percentage} / 100.0)$ multipliers.
* **Effective Defense**:
  * `Fortify` applies $(1.0 + \text{defense\_pct} / 100.0)$ multiplier.
* **Incoming Multiplier**:
  * `Vulnerability` multiplies incoming damage by $1.0 + \text{damage\_pct} / 100.0$.
* **Bleed Multiplier**:
  * If a one-shot `Bleed` effect is present on the attacker, it is consumed to multiply damage by $1.0 + \text{bleed\_damage\_pct} / 100.0$.
* **Critical Multiplier**:
  * Equals $2.0$ on a critical hit, and $1.0$ otherwise.
* **Minimum Damage**: Final damage is clamped to a minimum of $1.0$.

---

### 4. On-Hit and Reflection Effects

* **Lifesteal**: Heals the attacker by $\text{Final Damage} \times \sum (\text{Lifesteal\_pct} / 100.0)$.
* **Thorns**: Reflects damage back to the attacker, hitting them for $\text{Final Damage} \times \sum (\text{Thorns\_damage\_reflected\_pct} / 100.0)$.
* **Weapon Effects**: Applies active weapons' on-hit effect chains (e.g. Poison, Burn, Pierce) to the defender, and defensive weapon/shield on-being-hit effect chains back to the attacker.

