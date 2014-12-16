use std::fmt::Show;

use std::rc::{ Rc };
use std::cell::RefCell;

use anymap::AnyMap;

use entity::{ EntityManager };
use control::{ Control };

pub trait System<S> {
    fn update<A>(&mut self, entity_manager: &Rc<RefCell<EntityManager>>, &mut Control<S>, args: &A) where A: Show;
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

    pub fn register<S>(&mut self, system: S) where S: System<S> + 'static {
        self.systems.insert(system);
    }

    pub fn update<A, S>(&mut self, entity_manager: &Rc<RefCell<EntityManager>>, args: &A) where S: System<S> + 'static, A: Show {
        match self.systems.get_mut::<S>() {
            Some(system) => {
                let mut control: Control<S> = Control::new();
                system.update(entity_manager, &mut control, args);
                control.apply(entity_manager, system);
            },
            None => panic!("Tried to update unregistered system")
        }
    }
}