// Copyright 2016 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

//! # civng
//!
//! A Civ 5 implementation with a Text-based UI.
//!
//! See README for app-level details. This is simply an API documentation.

// This lib.rs unit is there so we can run doctests. There's a limitation on cargo where it can
// only run tests on libraries. See https://github.com/rust-lang/cargo/issues/1274
//
// This library is also the starting point for our API doc (not main.rs)
//

// Makes doctest so much simpler... Instead of having to declare extern crates in doctest, we
// simply import rustty stuff from the civng namespace.
pub use rustty::ui::{Widget};

extern crate num;
extern crate rustty;
extern crate byteorder;
extern crate rand;
#[macro_use]
extern crate bitflags;

pub mod hexpos;
pub mod terrain;
pub mod map;
pub mod unit;
pub mod combat;
pub mod screen;
pub mod civ5map;
pub mod selection;
pub mod game;
pub mod ai;
pub mod overhead;
pub mod details_window;
pub mod combat_confirm_dialog;
pub mod combat_result_window;
