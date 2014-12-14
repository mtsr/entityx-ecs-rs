#![macro_escape]
#![feature(macro_rules)]
extern crate anymap;

pub use entity::{ EntityManager, Entity, ComponentDatastructure };
pub use system::{ System, SystemManager };
pub use control::{ Control };

pub use tup_append::TupAppend;

pub mod tup_append;
pub mod macros;
pub mod system;
pub mod entity;
pub mod control;