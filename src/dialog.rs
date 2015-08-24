/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::ui::Window;

pub trait Dialog {
    fn window(&self) -> &Window;
    fn window_mut(&mut self) -> &mut Window;
}

