use std::collections::{ BinaryHeap, Bitv, VecMap, HashMap };
use std::collections::binary_heap;

use std::rc::{ Rc, Weak };
use std::cell::RefCell;
use std::iter::{ Iterator };
use std::intrinsics::TypeId;
use std::uint;

use anymap::AnyMap;

#[deriving(Show)]
pub struct EntityId {
    index: uint,
    version: uint
}

impl PartialEq for EntityId {
    fn eq(&self, other: &EntityId) -> bool {
        self.index == other.index && self.version == other.version
    }
}

pub struct Entity {
    id: EntityId,
    manager: Weak<RefCell<EntityManager>>,
}

impl<'a> Entity {
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

impl PartialEq for Entity {
    fn eq(&self, other: &Entity) -> bool {
        self.id() == other.id() && self.manager.upgrade() == other.manager.upgrade()
    }
}

pub type ComponentId = u64;

pub enum ComponentDatastructure {
    Vec,
    VecMap,
    HashMap,
}

pub struct EntityManager {
    next_entity_index: uint,
    free_entity_index_list: BinaryHeap<uint>,

    entity_versions: Vec<uint>,
    entity_component_masks: Vec<Bitv>,

    // TODO replace with HashMap<TypeId, Any>
    // Where Any is Vec<Option<C>> VecMap<Option<C>> or HashMap<Option<C>>
    // so that it's possible to access component lists without <C>
    // TODO Add BTreeMap
    component_lists: AnyMap,
    component_datastructures: HashMap<ComponentId, ComponentDatastructure>,
    component_index_counter: uint,
    component_indices: HashMap<ComponentId, uint>,

    weak_self: Option<Weak<RefCell<EntityManager>>>,
}

impl<'a> EntityManager {
    pub fn new() -> Rc<RefCell<EntityManager>> {
        // TODO see if Rc can be removed (replaced by Box?)
        let entity_manager = Rc::new(RefCell::new(EntityManager {
            next_entity_index: 0,
            free_entity_index_list: BinaryHeap::with_capacity(32),

            entity_versions: Vec::from_elem(256, 0),

            component_lists: AnyMap::new(),
            component_datastructures: HashMap::new(),

            component_index_counter: 0,
            entity_component_masks: Vec::with_capacity(256),
            component_indices: HashMap::with_capacity(32),

            weak_self: None,
        }));
        entity_manager.borrow_mut().weak_self = Some(entity_manager.downgrade());
        entity_manager
    }

    fn create_id(&mut self) -> EntityId {
        let index = match self.free_entity_index_list.pop() {
            Some(result) => result,
            None => {
                let result = self.next_entity_index;
                self.next_entity_index += 1;
                result
            }
        };
        let version = self.entity_versions[index];
        EntityId {
            index: index,
            version: version,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.entity_component_masks.push(Bitv::with_capacity(self.component_index_counter, false));
        Entity {
            id: self.create_id(),
            manager: self.weak_self.as_ref().unwrap().clone(),
        }
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entity_versions[entity.index()] += 1;
        self.free_entity_index_list.push(entity.index());
        self.entity_component_masks[entity.index()].clear();
    }

    fn is_valid(&self, entity: &Entity) -> bool {
        entity.index() < self.next_entity_index
        && entity.version() == self.entity_versions[entity.index()]
    }

    pub fn register_component<C: 'static>(&mut self, component_datastructure: ComponentDatastructure) {
        if self.component_lists.contains::<C>() {
            panic!("Tried to register component twice");
        }

        match component_datastructure {
            ComponentDatastructure::Vec => {
                let component_list: Vec<Option<C>> = Vec::from_fn(self.entity_versions.len(), |_| None);
                self.component_lists.insert::<Vec<Option<C>>>(component_list);
            },
            ComponentDatastructure::VecMap => {
                let component_list: VecMap<C> = VecMap::new();
                self.component_lists.insert::<VecMap<C>>(component_list);
            },
            ComponentDatastructure::HashMap => {
                let component_list: HashMap<uint, C> = HashMap::new();
                self.component_lists.insert::<HashMap<uint, C>>(component_list);
            },
        }

        self.component_datastructures.insert(TypeId::of::<C>().hash(), component_datastructure);

        self.component_indices.insert(TypeId::of::<C>().hash(), self.component_index_counter);
        self.component_index_counter += 1;
        let length = self.component_index_counter;

        for mut entity_component_mask in self.entity_component_masks.iter_mut() {
            // dynamically grow bitv length, only needed if new component types can be registered after entities have been added
            entity_component_mask.grow(length, false);
        }
    }

    pub fn assign_component<C: 'static>(&mut self, entity: &Entity, component: C) {
        assert!(self.is_valid(entity));

        match self.component_datastructures.get(&TypeId::of::<C>().hash()) {
            Some(&ComponentDatastructure::Vec) => {
                match self.component_lists.get_mut::<Vec<Option<C>>>() {
                    Some(component_list) => {
                        component_list[entity.index()] = Some(component);
                    },
                    None => panic!("Tried to assign unregistered component"),
                };
            },
            Some(&ComponentDatastructure::VecMap) => {
                match self.component_lists.get_mut::<VecMap<C>>() {
                    Some(component_list) => {
                        component_list.insert(entity.index(), component);
                    },
                    None => panic!("Tried to assign unregistered component"),
                };
            },
            Some(&ComponentDatastructure::HashMap) => {
                match self.component_lists.get_mut::<HashMap<uint, C>>() {
                    Some(component_list) => {
                        component_list.insert(entity.index(), component);
                    },
                    None => panic!("Tried to assign unregistered component"),
                };
            },
            None => panic!("Tried to assign unregistered component"),
        }
        match self.component_indices.get(&TypeId::of::<C>().hash()) {
            Some(index) => self.entity_component_masks[entity.index()].set(*index, true),
            None => panic!("Tried to assign unregistered component"),
        };
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity) -> bool {
        assert!(self.is_valid(entity));
        match self.component_indices.get(&TypeId::of::<C>().hash()) {
            Some(index) => self.entity_component_masks[entity.index()][*index],
            None => panic!("Tried to check for unregistered component"),
        }
    }

    // TODO dedup get_component and get_component_mut
    pub fn get_component<C: 'static>(&'a self, entity: &Entity) -> Option<&C> {
        assert!(self.is_valid(entity));
        match self.component_indices.get(&TypeId::of::<C>().hash()) {
            Some(index) => {
                if !self.entity_component_masks[entity.index()][*index] {
                    return None;
                }
            },
            None => panic!("Tried to get unregistered component"),
        }
        match self.component_datastructures.get(&TypeId::of::<C>().hash()) {
            Some(&ComponentDatastructure::Vec) => {
                match self.component_lists.get::<Vec<Option<C>>>() {
                    Some(component_list) => {
                        component_list[entity.index()].as_ref()
                    },
                    None => panic!("Tried to get unregistered component"),
                }
            },
            Some(&ComponentDatastructure::VecMap) => {
                match self.component_lists.get::<VecMap<C>>() {
                    Some(component_list) => {
                        component_list.get(&entity.index())
                    },
                    None => panic!("Tried to get unregistered component"),
                }
            },
            Some(&ComponentDatastructure::HashMap) => {
                match self.component_lists.get::<HashMap<uint, C>>() {
                    Some(component_list) => {
                        component_list.get(&entity.index())
                    },
                    None => panic!("Tried to get unregistered component"),
                }
            },
            None => panic!("Tried to assign unregistered component"),
        }
    }

    pub fn get_component_mut<C: 'static>(&'a mut self, entity: &Entity) -> Option<&mut C> {
        assert!(self.is_valid(entity));
        match self.component_indices.get_mut(&TypeId::of::<C>().hash()) {
            Some(index) => {
                if !self.entity_component_masks[entity.index()][*index] {
                    return None;
                }
            },
            None => panic!("Tried to get unregistered component"),
        }
        match self.component_datastructures.get(&TypeId::of::<C>().hash()) {
            Some(&ComponentDatastructure::Vec) => {
                match self.component_lists.get_mut::<Vec<Option<C>>>() {
                    Some(component_list) => {
                        component_list[entity.index()].as_mut()
                    },
                    None => panic!("Tried to get unregistered component"),
                }
            },
            Some(&ComponentDatastructure::VecMap) => {
                match self.component_lists.get_mut::<VecMap<C>>() {
                    Some(component_list) => {
                        component_list.get_mut(&entity.index())
                    },
                    None => panic!("Tried to get unregistered component"),
                }
            },
            Some(&ComponentDatastructure::HashMap) => {
                match self.component_lists.get_mut::<HashMap<uint, C>>() {
                    Some(component_list) => {
                        component_list.get_mut(&entity.index())
                    },
                    None => panic!("Tried to get unregistered component"),
                }
            },
            None => panic!("Tried to assign unregistered component"),
        }    }

    pub fn entities(&self) -> EntityIterator {
        EntityIterator {
            entity_manager: self.weak_self.as_ref().unwrap().clone(),
            next_entity_index: self.next_entity_index,
            index: 0,
            free_entity_index_list: self.free_entity_index_list.iter(),
        }
    }
}

impl PartialEq for EntityManager {
    fn eq(&self, other: &EntityManager) -> bool {
        self == other
    }
}

pub struct EntityIterator<'a> {
    entity_manager: Weak<RefCell<EntityManager>>,
    next_entity_index: uint,
    index: uint,
    free_entity_index_list: binary_heap::Items<'a, uint>,
}

impl<'a> Iterator<Entity> for EntityIterator<'a> {
    fn next(&mut self) -> Option<Entity> {
        while self.index < self.next_entity_index {
            let mut free_entity_index = -1;

            while free_entity_index < self.index {
                free_entity_index = match self.free_entity_index_list.next() {
                    Some(x) => *x,
                    None => uint::MAX,
                }
            }

            if free_entity_index == self.index {
                self.index += 1;
                continue;
            }

            let version = self.entity_manager.upgrade().unwrap().borrow().entity_versions[self.index];

            let result = Some(Entity {
                id: EntityId {
                    index: self.index,
                    version: version,
                },
                manager: self.entity_manager.clone(),
            });

            self.index += 1;
            return result;
        }

        None
    }
}