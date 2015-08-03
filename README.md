# civng

*Civ 5 implementation in Rust with a text-based UI, because, why not?*

Were you ever like "God is Civ 5 slow to load!" and wished you could play the game without fancy
graphics? Or wished you could play Civ on your small netbook? Yeah, me neither, but I needed a
reason to dash head-first into this folly.

Ok, I know, I'm crazy and I'm never going to finish this, but then, *what the hell* it's so much
fun to implement.

## Current status

Very early. Features:

* Text-based hexagonal tiles UI.
* Loads ".Civ5Map" files.
* Move around a cute `X` unit.
* Basic terrain types, with some of them impassable.
* Map scrolling.

## Requirements

* [Rust][rust] 1.1
* [Cargo][cargo]
* A terminal using a font that supports [Unicode box-drawing characters][boxdrawing]

## Build

To build `civng`, make sure you have all requirements, then do:

    git clone https://github.com/hsoft/civng.git
    cd civng
    cargo build
    ./target/debug/civng

You have to run the executable at the root of the project because paths for some needed resources
are hardcoded.

### Tests & documentation

There are a couple of doctests which you can run with:

    cargo test

You can also generate an API documentation with:

    cargo doc

and then open `target/doc/civng/index.html`.

## Hex cells orientation

In Civ 5, hex cells are "pointy topped", but in `civng`, our cells are "flat topped". This is
because it's much harder to ascii-draw a good-looking pointy-topped cell than a flat topped one.
This changes significantly how maps look, but it shouldn't affect gameplay.

## Usage

The app starts with the top left cell of the screen being the top left cell of the map. There's a
`X` unit on it. You can move it with the keypad, but make sure you have numlock enabled.

Water `~` and mountains `A` are impassable, except at the beginning because our `X` unit starts on
water. Until the `X` unit reaches passable terrain, it can go anywhere. After that, it can't pass
through impassable terrain.

You can toggle position markers with `p`.

You can scroll the map! To do so, press `s` to toggle scroll mode. Now, when you press numpad keys,
it's the map that will scroll instead of unit movement.

[rust]: http://www.rust-lang.org/
[cargo]: https://crates.io/
[boxdrawing]: https://en.wikipedia.org/wiki/Box-drawing_character

