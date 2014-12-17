use std::fmt::Show;

use std::rc::{ Rc };
use std::cell::RefCell;

use anymap::AnyMap;

use entity::{ EntityManager };
use control::{ Control };

pub trait System<Id, S> {
    fn update<A>(&mut self, entity_manager: &EntityManager<Id>, &mut Control<Id, S>, args: &A) where A: Show;
}

pub struct SystemManager<Id> {
    systems: AnyMap
}

impl<Id> SystemManager<Id> {
    pub fn new() -> SystemManager<Id> {
        SystemManager {
            systems: AnyMap::new()
        }
    }

    pub fn register<S>(&mut self, system: S) where S: System<Id, S> + 'static {
        self.systems.insert(system);
    }

    pub fn update<A, S>(&mut self, entity_manager: &mut EntityManager<Id>, args: &A) where S: System<Id, S> + 'static, A: Show {
        match self.systems.get_mut::<S>() {
            Some(system) => {
                let mut control: Control<Id, S> = Control::new();
                system.update(entity_manager, &mut control, args);
                control.apply(entity_manager, system);
            },
            None => panic!("Tried to update unregistered system")
        }
    }
}