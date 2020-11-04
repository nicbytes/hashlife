// pub mod linear;
// pub mod hashlife;
pub mod automata;
pub mod quad2x2;
pub mod quad4x4;
pub mod quad;
pub mod factory;

use std::rc::Rc;
pub type ReferenceCounter<T> = Rc<T>;
