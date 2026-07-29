# Quickstart

quest-rs is a 2D video game written in Rust. It uses SDL via the halcyon-rs crate,
which provides bindings; see `halcyon-rs/AGENTS.md` for more info.

# Core structs

Video games are complex. Rust complicates things further, since it imposes strict requirements on ownership.
As such, structs are often split up in potentially confusing ways.

All paths mentioned below are relative to `src/`.

## `Game` (`game/mod.rs`)

The top-level struct. Contains the main loop logic.

## `Atlas` (`atlas/mod.rs`)

A [texture atlas](https://en.wikipedia.org/wiki/Texture_atlas). Offers a simple API
for adding, removing, and replacing textures. Users interface with the atlas using an `AtlasId`,
which represents a texture "slot" on the atlas.

## `Resources` (`game/resources.rs`)

The actual window, renderer, etc. Is often passed as an argument whenever a function/method
has a need to manipulate the game, for example, to add a texture to the atlas.
