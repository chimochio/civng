# civng

*Civ 5 implementation in Rust with a text-based UI, because, why not?*

Were you ever like "God is Civ 5 slow to load!" and wished you could play the game without fancy
graphics? Or wished you could play Civ on your small netbook? Yeah, me neither, but I needed a
reason to dash head-first into this folly.

Ok, I know, I'm crazy and I'm never going to finish this, but then, *what the hell* it's so much
fun to implement.

[![asciinema](https://asciinema.org/a/ecps7blbb5l65isrov4x6c419.png)](https://asciinema.org/a/ecps7blbb5l65isrov4x6c419)

## Current status

Very early. Features:

* Text-based hexagonal tiles UI.
* Loads ".Civ5Map" files.
* Move around Larry (`L`) and Benny (`B`).
* Combat! Meet Evil Man (`E`), an enemy unit shown in red somewhere on the map and fight him!
* Nice pre/post combat dialogs with neat info.
* Some combat modifiers, such as terrain and flanking.
* Basic terrain types, with some of them impassable.
* Map scrolling.
* Turns, with limited movements in each one.
* Active unit is colored.

## Requirements

* [Rust][rust] 1.2
* [Cargo][cargo]
* A terminal using a font that supports [Unicode box-drawing characters][boxdrawing]

Dependencies (automatically installed by cargo):

* [Rustty][rustty] for terminal output.
* [num][num] because native integers have very limited capabilities.
* [byteorder][byteorder] for Civ5Map format destructuring.

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

The app starts with the top left cell of the screen being the top left cell of the map. There are
two units, Larry and Benny (`L` and `B`) which move in turns. You can move it with `w/a/s/d/q/e`.

Each unit has two movements per turn. You can cycle through active units with `.`. You start a new
turn by pressing Return.

If you move on a tile with an enemy unit on it, you initiate a melee attack automatically. If you
win, you move on the enemy's tile.

Water `~` and mountains `A` are impassable. Hills `^` use up 2 movement points.

You can toggle position markers (mostly for debugging purposes) with `shift-p`.

You can scroll the map! To do so, press `shift-s` to toggle scroll mode. Now, when you press
movement keys, it's the map that will scroll instead of the active unit.

`shift-q` to quit.

[rust]: http://www.rust-lang.org/
[cargo]: https://crates.io/
[boxdrawing]: https://en.wikipedia.org/wiki/Box-drawing_character
[rustty]: https://github.com/cpjreynolds/rustty
[num]: https://crates.io/crates/num
[byteorder]: https://crates.io/crates/byteorder

