extern crate anymap;

use anymap::AnyMap;

pub trait System {
    fn update<A>(&self, entities: &EntityManager, args: A);
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

    pub fn register<S: System + 'static>(&mut self, system: Box<S>) {
        self.systems.insert(*system);
    }

    pub fn update<S: System + 'static, A>(&self, entities: &EntityManager, args: A) {
        match self.systems.get::<S>() {
            Some(system) => {
                system.update(entities, args);
            },
            None => panic!("Tried to update unregistered system")
        }
    }
}

pub struct Entity<'a> {
    index: u32,
    version: u32,
    // removed because it would prevent other &mut access 
    // manager: &EntityManager
}

impl<'a> Entity<'a> {
    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

    pub fn get_id(&self) -> u64 {
        self.version as u64 << 32u & self.index as u64
    }
}

// impl<'a> PartialEq for Entity<'a> {
//     fn eq(&self, other: &Entity) -> bool {
//         self.index == other.index && self.version == other.version && self.manager == other.manager
//     }
// }

pub struct EntityManager {
    index_counter: u32,
    free_index_list: Vec<u32>,

    entity_version: Vec<u32>,

    components: AnyMap,
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            index_counter: 0,
            free_index_list: Vec::with_capacity(32),

            entity_version: Vec::from_elem(256, 0),

            components: AnyMap::new()
        }
    }

    pub fn create(&mut self) -> Entity {
        let index = match self.free_index_list.pop() {
            Some(result) => result,
            None => {
                self.index_counter += 1;
                self.index_counter - 1
            }
        };
        let version = self.entity_version[index as uint];
        Entity {
            index: index,
            version: version,
            // manager: self
        }
    }

    pub fn register_component<C: 'static>(&mut self) {
        if self.components.contains::<C>() {
            panic!("Tried to register component twice");
        }
        let component_list: Vec<C> = Vec::with_capacity(self.entity_version.len());
        self.components.insert::<Vec<C>>(component_list);
    }

    pub fn assign<C: 'static>(&mut self, entity: &Entity, component: C) {
        match self.components.get_mut::<Vec<C>>() {
            Some(component_list) => {
                component_list[entity.get_index() as uint] = component;
            },
            None => panic!("Tried to assign unregistered component")
        };
    }
}

impl PartialEq for EntityManager {
    fn eq(&self, other: &EntityManager) -> bool {
        self == other
    }
}