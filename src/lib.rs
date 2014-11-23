extern crate anymap;

use anymap::AnyMap;

use std::rc::{ Rc, Weak };
use std::cell::RefCell;

pub trait System {
    fn update<A>(&self, entities: Rc<RefCell<EntityManager>>, args: A);
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

    pub fn update<S: System + 'static, A>(&self, entities: Rc<RefCell<EntityManager>>, args: A) {
        match self.systems.get::<S>() {
            Some(system) => {
                system.update(entities, args);
            },
            None => panic!("Tried to update unregistered system")
        }
    }
}

pub struct EntityId {
    index: u32,
    version: u32
}

impl PartialEq for EntityId {
    fn eq(&self, other: &EntityId) -> bool {
        self.index == other.index && self.version == other.version
    }
}

pub struct Entity<'a> {
    id: EntityId,
    manager: Weak<RefCell<EntityManager>>,
}

impl<'a> Entity<'a> {
    pub fn new(manager: Weak<RefCell<EntityManager>>) -> Entity<'a> {
        Entity {
            // TODO unwrap unsafe?
            id: manager.upgrade().unwrap().borrow_mut().create_id(),
            manager: manager,
        }
    }

    pub fn assign<C: 'static + Clone + Copy>(&self, component: C) {
        self.manager.upgrade().unwrap().borrow_mut().assign_component(self, component);
    }

    pub fn index(&self) -> u32 {
        self.id.index
    }

    pub fn version(&self) -> u32 {
        self.id.version
    }

    pub fn id(&self) -> EntityId {
        self.id
    }
}

impl<'a> PartialEq for Entity<'a> {
    fn eq(&self, other: &Entity) -> bool {
        // TODO upgrade really needed?
        self.id() == other.id() && self.manager.upgrade() == other.manager.upgrade()
    }
}

pub struct EntityManager {
    index_counter: u32,
    free_index_list: Vec<u32>,

    entity_version: Vec<u32>,

    components: AnyMap,
}

impl EntityManager {
    pub fn new() -> Rc<RefCell<EntityManager>> {
        Rc::new(RefCell::new(EntityManager {
            index_counter: 0,
            free_index_list: Vec::with_capacity(32),

            entity_version: Vec::from_elem(256, 0),

            components: AnyMap::new()
        }))
    }

    pub fn create_id(&mut self) -> EntityId {
        let index = match self.free_index_list.pop() {
            Some(result) => result,
            None => {
                self.index_counter += 1;
                self.index_counter - 1
            }
        };
        let version = self.entity_version[index as uint];
        EntityId {
            index: index,
            version: version,
        }
    }

    pub fn register_component<C: 'static + Clone + Copy>(&mut self) {
        if self.components.contains::<C>() {
            panic!("Tried to register component twice");
        }
        let component_list: Vec<C> = Vec::with_capacity(self.entity_version.len());
        self.components.insert::<Vec<C>>(component_list);
    }

    pub fn assign_component<C: 'static + Clone + Copy>(&mut self, entity: &Entity, component: C) {
        match self.components.get_mut::<Vec<C>>() {
            Some(component_list) => {
                // ensure component_list size by adding copies of component
                // require component to be Clone + Copy
                if component_list.len() <= entity.index() as uint {
                    component_list.grow(self.entity_version.len(), component);
                }

                component_list[entity.index() as uint] = component;
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