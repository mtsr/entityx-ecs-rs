use anymap::AnyMap;

use entity::{ EntityManager };
use component::{ ComponentManager };
use control::{ Control };

pub trait System<WorldId, S> {
    fn update<A>(&mut self, entity_manager: &EntityManager<WorldId>, component_manager: &ComponentManager<WorldId>, &mut Control<WorldId, S>, args: &A);
}

pub struct SystemManager<WorldId> {
    systems: AnyMap
}

impl<WorldId> SystemManager<WorldId> {
    pub fn new() -> SystemManager<WorldId> {
        SystemManager {
            systems: AnyMap::new()
        }
    }

    pub fn register<S>(&mut self, system: S) where S: System<WorldId, S> + 'static {
        self.systems.insert(system);
    }

    pub fn update<A, S>(&mut self, entity_manager: &mut EntityManager<WorldId>, component_manager: &ComponentManager<WorldId>, args: &A) where S: System<WorldId, S> + 'static {
        match self.systems.get_mut::<S>() {
            Some(system) => {
                let mut control: Control<WorldId, S> = Control::new();
                system.update(entity_manager, component_manager, &mut control, args);
                control.apply(entity_manager, system);
            },
            None => panic!("Tried to update unregistered system")
        }
    }
}