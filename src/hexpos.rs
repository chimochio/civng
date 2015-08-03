/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Pure (UI-less) hex cell positioning logic. Inspired from this
//! [awesome resrouce on the subject](http://www.redblobgames.com/grids/hexagons/).
//!
//! The vocabulary here is entirely borrowed from that referenced article, so you can lookup there
//! for reference.
//!
//! Note that this module assumes a "flat-topped" hex grid.
//!
//! `i32` is chosen as a base integer type because positions in hex grids often have to go negative
//! even with a top-left origin.

use num::integer::Integer;

/// Possible move directions in a flat-topped hex grid
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

/// "Cube"-type position. We simply call it `Pos` for conciseness because that's our "official" pos.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32, z: i32) -> Pos {
        Pos {
            x: x,
            y: y,
            z: z,
        }
    }

    /// Returns pos `(0, 0, 0)`
    pub fn origin() -> Pos {
        Pos::new(0, 0, 0)
    }

    pub fn to_axialpos(&self) -> AxialPos {
        AxialPos::new(self.x, self.z)
    }

    pub fn to_offset_pos(&self) -> OffsetPos {
        // Each x means +1x, -½y, -½z. y goes first.
        // Each y means -1y, +1z.
        let x = self.x;
        let y = self.z + self.x / 2;
        OffsetPos::new(x, y)
    }

    /// Translates `self` by `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::hexpos::{Pos, Direction};
    ///
    /// let pos1 = Pos::origin().neighbor(Direction::South);
    /// let pos2 = Pos::origin().neighbor(Direction::SouthWest);
    /// let pos3 = Pos::origin().neighbor(Direction::South).neighbor(Direction::SouthWest);
    /// assert_eq!(pos1.translate(pos2), pos3);
    /// ```
    pub fn translate(&self, other: Pos) -> Pos {
        Pos::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }

    /// Multiplies `self` by `factor`.
    ///
    /// This increases the distance to our origin by that factor.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::hexpos::{Pos, Direction};
    ///
    /// let mut pos1 = Pos::origin();
    /// for _ in 0..3 {
    ///     pos1 = pos1.neighbor(Direction::South);
    /// }
    /// let pos2 = Pos::origin().neighbor(Direction::South).amplify(3);
    /// assert_eq!(pos1, pos2);
    /// ```
    pub fn amplify(&self, factor: i32) -> Pos {
        Pos::new(
            self.x * factor,
            self.y * factor,
            self.z * factor,
        )
    }

    /// Returns cell at the opposite side of the origin.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::hexpos::{Pos, Direction};
    ///
    /// let pos1 = Pos::origin().neighbor(Direction::South).neighbor(Direction::SouthWest);
    /// let pos2 = Pos::origin().neighbor(Direction::North).neighbor(Direction::NorthEast);
    /// assert_eq!(pos1, pos2.neg());
    /// ```
    pub fn neg(&self) -> Pos {
        Pos::new(
            -self.x,
            -self.y,
            -self.z,
        )
    }

    /// Returns a pos relative to `self` when moving in the specified `direction`.
    ///
    /// By "moving", we mean moving a distance of a single cell.
    pub fn neighbor(&self, direction: Direction) -> Pos {
        let mut p = *self;
        match direction {
            Direction::North => { p.y += 1; p.z -= 1 },
            Direction::NorthEast => { p.x += 1; p.z -= 1 },
            Direction::SouthEast => { p.x += 1; p.y -= 1 },
            Direction::South => { p.z += 1; p.y -= 1 },
            Direction::SouthWest => { p.z += 1; p.x -= 1 },
            Direction::NorthWest => { p.y += 1; p.x -= 1 },
        }
        p
    }

    pub fn fmt(&self) -> String {
        format!("{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(Copy, Clone)]
pub struct AxialPos {
    pub q: i32,
    pub r: i32,
}

impl AxialPos {
    pub fn new(q: i32, r: i32) -> AxialPos {
        AxialPos {
            q: q,
            r: r,
        }
    }

    pub fn to_pos(&self) -> Pos {
        Pos::new(self.q, self.r - self.q, self.r)
    }

    pub fn fmt(&self) -> String {
        format!("{},{}", self.q, self.r)
    }
}

/// "odd-q" type of Offset position.
///
/// Origin is top-left. `(1, 0)` is SouthEast of origin. `(0, 1)` is South.
#[derive(Copy, Clone)]
pub struct OffsetPos {
    pub x: i32,
    pub y: i32,
}

impl OffsetPos {
    pub fn new(x: i32, y: i32) -> OffsetPos {
        OffsetPos {
            x: x,
            y: y,
        }
    }

    pub fn to_pos(&self) -> Pos {
        // Each x means +1x, -½y, -½z. y goes first.
        // Each y means -1y, +1z.
        let (halfx, remx) = self.x.div_rem(&2);
        let x = self.x;
        let y = (-self.y) + (-halfx - remx);
        let z = self.y + (-halfx);
        Pos::new(x, y, z)
    }

    pub fn fmt(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
}

