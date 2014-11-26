extern crate anymap;

use anymap::AnyMap;

use std::rc::{ Rc, Weak };
use std::cell::RefCell;
use std::iter::{ Range, Iterator, Filter, Map };
use std::slice::Items;
use std::collections::{ BinaryHeap, Bitv, HashMap };
use std::uint;
use std::intrinsics::TypeId;

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
    index: uint,
    version: uint
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
    pub fn assign<C: 'static>(&self, component: C) {
        self.manager.upgrade().unwrap().borrow_mut().assign_component(self, component);
    }

    pub fn index(&self) -> uint {
        self.id.index
    }

    pub fn version(&self) -> uint {
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
    entity_index_counter: uint,
    free_index_list: BinaryHeap<uint>,

    entity_version: Vec<uint>,

    components: AnyMap,

    component_index_counter: uint,
    components_mask: Vec<Bitv>,
    component_indices: HashMap<u64, uint>,

    weak_self: Option<Weak<RefCell<EntityManager>>>,
}

impl EntityManager {
    pub fn new() -> Rc<RefCell<EntityManager>> {
        let entity_manager = Rc::new(RefCell::new(EntityManager {
            entity_index_counter: 0,
            free_index_list: BinaryHeap::with_capacity(32),

            entity_version: Vec::from_elem(256, 0),

            components: AnyMap::new(),

            component_index_counter: 0,
            components_mask: Vec::with_capacity(256),
            component_indices: HashMap::with_capacity(32),

            weak_self: None,
        }));
        entity_manager.borrow_mut().weak_self = Some(entity_manager.downgrade());
        entity_manager
    }

    fn create_id(&mut self) -> EntityId {
        let index = match self.free_index_list.pop() {
            Some(result) => result,
            None => {
                self.entity_index_counter += 1;
                self.entity_index_counter - 1
            }
        };
        let version = self.entity_version[index];
        EntityId {
            index: index,
            version: version,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.components_mask.push(Bitv::with_capacity(self.component_index_counter, false));
        Entity {
            // TODO unwrap unsafe?
            id: self.create_id(),
            manager: self.weak_self.as_ref().unwrap().clone(),
        }
    }

    pub fn register_component<C: 'static>(&mut self) {
        if self.components.contains::<C>() {
            panic!("Tried to register component twice");
        }
        let component_list: Vec<Option<C>> = Vec::from_fn(self.entity_version.len(), |index| None);
        self.components.insert::<Vec<Option<C>>>(component_list);

        self.component_indices.insert(TypeId::of::<C>().hash(), self.component_index_counter);
        self.component_index_counter += 1;
        let length = self.component_index_counter;

        for mut entity_component_mask in self.components_mask.iter_mut() {
            // dynamically grow bitv length, only needed if new components can be registered later
            entity_component_mask.grow(length, false);
        }
    }

    pub fn assign_component<C: 'static>(&mut self, entity: &Entity, component: C) {
        match self.components.get_mut::<Vec<Option<C>>>() {
            Some(component_list) => {
                component_list[entity.index()] = Some(component);
            },
            None => panic!("Tried to assign unregistered component")
        };
        match self.component_indices.get(&TypeId::of::<C>().hash()) {
            Some(index) => self.components_mask[entity.index()].set(*index, true),
            None => panic!("Tried to assign unregistered component")
        };
    }

    pub fn entities_with_component<C: 'static>(&self) -> Map<&Option<C>, &C, Filter<&Option<C>, Items<Option<C>>>> {

        match self.components.get::<Vec<Option<C>>>() {
            Some(component_list) => {
                component_list.iter().filter(|o| o.is_some()).map(|o| o.as_ref().unwrap())
            },
            None => panic!("Tried to get unregistered component")
        }
    }
}

impl PartialEq for EntityManager {
    fn eq(&self, other: &EntityManager) -> bool {
        self == other
    }
}
