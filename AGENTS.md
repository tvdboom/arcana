# Arcana Agent Guide

Arcana is a Bevy 0.19, Rust 2021 dark-fantasy RPG for native and WebAssembly. The core loop
spends action points on progression, gear, and preparation before real-time PvE/PvP combat.

## Project map

- `src/main.rs`: application/plugin setup.
- `src/core/actions/`: Rest, Study, Work, Train, Craft, Shop, Hunt, Quest, and Duel flows.
- `src/core/combat/`: simulation and combat UI. Keep game rules in `mechanics.rs`.
- `src/core/catalog/`: serialized catalog types, lookup API, and invariant tests.
- `src/core/ui/`, `menu/`: Bevy UI construction and interaction systems.
- `src/core/player.rs`, `persistence.rs`, `network.rs`: character state, saves, and native PvP.
- `src/bin/generate_catalogs.rs`: deterministic source of catalog names, levels, prices,
  stats, effects, and monster progression.
- `assets-src/`: source assets. `build.rs` produces runtime `assets/`; `pack-assets` creates
  shipping archives. Generated `assets/`, `target/`, and most `docs/` output are ignored.

## Working rules

- Change catalog logic in `src/bin/generate_catalogs.rs`; never hand-edit generated RON.
- Preserve deterministic generation: sort inputs and derive variants from stable indexes.
  Do not randomly shuffle progression-sensitive content.
- Catalog levels stay in `1..=20`. Names must be unique within each catalog and across all
  equipment catalogs. Every image key must resolve to a generated asset.
- Keep an item's name, icon, kind, effects, price, and stats semantically aligned. Mundane
  food/materials begin cheap; power, durability, rarity, and price should scale together.
- Physical/magical ability availability should remain broadly balanced. Ability cooldowns
  must outlast their longest timed effect, and `on_self` must match the effect targets.
- Shields and books do not auto-attack. Two-handed weapons trade speed/flexibility for power;
  dual-wielding must not duplicate the character's shared attack contribution.
- Monster health, attack, defense, initiative, regeneration, rewards, and effects must scale
  by level. Only generate age/stage names for which the artwork actually exists.
- When serialized save fields change, test backward loading and add a versioned migration;
  bincode layout changes are not made safe by a Serde default alone.
- Keep native-only networking, dialogs, and asset conversion behind `cfg(not(wasm32))`.

## Rust conventions

- Prefer typed enums/newtypes and exhaustive `match` expressions over stringly game logic.
- Keep Bevy systems focused: query narrowly, return early when state is irrelevant, and use
  resources/messages for cross-system communication.
- Use checked/saturating arithmetic at inventory, currency, AP, XP, and level boundaries.
- Return `Result` for recoverable I/O/network failures; reserve `expect`/`unwrap` for static
  catalog/assets whose validity is covered by startup or tests.
- Avoid `unsafe`. Reuse shared helpers, keep calculations pure where practical, and add a
  regression test with every combat or catalog bug fix.
- Document every function with a concise `///` summary. Add a second paragraph and parameter
  notes only when the contract, targeting, units, or side effects are not obvious.

## Before handing off

Run:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

For catalog changes, also run `cargo run --bin generate_catalogs` and ensure catalog invariant
tests pass. For asset-pipeline changes, run `cargo run --bin build-assets`; use
`cargo run --release --bin pack-assets` only when shipping archives are needed.
