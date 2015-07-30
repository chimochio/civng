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
* Move around a cute `X` unit with the numpad (have numlock on!).
* Basic terrain types, with some of them impassable.

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

[rust]: http://www.rust-lang.org/
[cargo]: https://crates.io/
[boxdrawing]: https://en.wikipedia.org/wiki/Box-drawing_character

