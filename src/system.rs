use std::fmt::Show;

use std::rc::{ Rc };
use std::cell::RefCell;

use anymap::AnyMap;

use entity::{ EntityManager };
use control::{ Control };

pub trait System {
    fn update<A>(&mut self, entity_manager: &Rc<RefCell<EntityManager>>, &mut Control, args: &A) where A: Show;
}

pub struct SystemManager {
    systems: AnyMap
}

impl SystemManager {
    pub fn new() -> SystemManager {
        SystemManager {
            systems: AnyMap::new()
        }
    }

    pub fn register<S>(&mut self, system: S) where S: System + 'static {
        self.systems.insert(system);
    }

    pub fn update<A, S>(&mut self, entity_manager: &Rc<RefCell<EntityManager>>, args: &A) where S: System + 'static, A: Show {
        match self.systems.get_mut::<S>() {
            Some(system) => {
                let mut control = Control::new();
                system.update(entity_manager, &mut control, args);
                control.apply(entity_manager);
            },
            None => panic!("Tried to update unregistered system")
        }
    }
}