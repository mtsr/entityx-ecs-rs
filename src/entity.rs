use std::collections::{ BinaryHeap, Bitv, VecMap, HashMap };
use std::collections::binary_heap;
// TODO Consider using unsafe for transmuting Option
// use std::mem::transmute;

use std::iter::{ Iterator };
use std::intrinsics::TypeId;
use std::uint;

use std::kinds::marker;

// TODO more DB like approach to ECS i.e. more powerful query tools
// TODO optimize getting entities with components
// by starting from the 'with' component with the fewest instances

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

pub struct Entity<Id> {
    id: EntityId,
    marker: marker::InvariantType<Id>,
}

impl<'a, Id> Entity<Id> {
    #[inline]
    pub fn index(&self) -> uint {
        self.id.index
    }

    #[inline]
    pub fn version(&self) -> uint {
        self.id.version
    }

    #[inline]
    pub fn id(&self) -> EntityId {
        self.id
    }
}

// impl PartialEq for Entity {
//     fn eq(&self, other: &Entity) -> bool {
//         // self.id() == other.id() && self.manager.upgrade() == other.manager.upgrade()
//     }
// }

pub type ComponentId = u64;

pub enum ComponentDatastructure {
    // TODO consider adding Vec<C> (without Option) for C: Default
    // Vec,
    VecMap,
    HashMap,
}

pub struct EntityManager<Id> {
    marker: marker::InvariantType<Id>,

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
}

impl<'a, Id> EntityManager<Id> {
    pub fn new() -> EntityManager<Id> {
        EntityManager {
            marker: marker::InvariantType,

            next_entity_index: 0,
            free_entity_index_list: BinaryHeap::with_capacity(32),

            entity_versions: Vec::from_elem(256, 0u),

            component_lists: AnyMap::new(),
            component_datastructures: HashMap::new(),

            component_index_counter: 0,
            entity_component_masks: Vec::with_capacity(256),
            component_indices: HashMap::with_capacity(32),
        }
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

        let capacity = self.entity_versions.capacity();
        if index >= capacity {
            self.entity_versions.grow(capacity * 2, 0u);
        }

        let version = self.entity_versions[index];
        EntityId {
            index: index,
            version: version,
        }
    }

    pub fn create_entity(&mut self) -> Entity<Id> {
        self.entity_component_masks.push(Bitv::with_capacity(self.component_index_counter, false));
        Entity {
            id: self.create_id(),
            marker: self.marker,
        }
    }

    pub fn destroy_entity(&mut self, entity: Entity<Id>) {
        self.entity_versions[entity.index()] += 1;
        self.free_entity_index_list.push(entity.index());
        self.entity_component_masks[entity.index()].clear();
    }

    pub fn is_valid(&self, entity: &Entity<Id>) -> bool {
        entity.index() < self.next_entity_index
        && entity.version() == self.entity_versions[entity.index()]
    }

    // TODO look into moving datastructure type into type parameter
    pub fn register_component<C: 'static>(&mut self, component_datastructure: ComponentDatastructure) {
        match component_datastructure {
            ComponentDatastructure::VecMap => {
                if self.component_lists.contains::<VecMap<C>>() {
                    panic!("Tried to register component twice");
                }
                let component_list: VecMap<C> = VecMap::new();
                self.component_lists.insert::<VecMap<C>>(component_list);
            },
            ComponentDatastructure::HashMap => {
                if self.component_lists.contains::<HashMap<uint, C>>() {
                    panic!("Tried to register component twice");
                }
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

    /// Add or replace component on entity
    pub fn assign_component<C: 'static>(&mut self, entity: &Entity<Id>, component: C) {
        assert!(self.is_valid(entity));

        match self.component_datastructures.get(&TypeId::of::<C>().hash()) {
            Some(&ComponentDatastructure::VecMap) => {
                // TODO Consider replacing with unsafe
                // let component_list: &mut VecMap<C> = unsafe { transmute(self.component_lists.get_mut::<VecMap<C>>()) };
                let component_list = self.component_lists.get_mut::<VecMap<C>>().unwrap();
                component_list.insert(entity.index(), component);
            },
            Some(&ComponentDatastructure::HashMap) => {
                let component_list = self.component_lists.get_mut::<HashMap<uint, C>>().unwrap();
                component_list.insert(entity.index(), component);
            },
            None => panic!("Tried to assign unregistered component"),
        }
        match self.component_indices.get(&TypeId::of::<C>().hash()) {
            Some(index) => self.entity_component_masks[entity.index()].set(*index, true),
            None => panic!("Tried to assign unregistered component"),
        };
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity<Id>) -> bool {
        assert!(self.is_valid(entity));

        match self.component_indices.get(&TypeId::of::<C>().hash()) {
            Some(index) => self.entity_component_masks[entity.index()][*index],
            None => panic!("Tried to check for unregistered component"),
        }
    }

    // TODO dedup get_component and get_component_mut
    pub fn get_component<C: 'static>(&'a self, entity: &Entity<Id>) -> Option<&C> {
        assert!(self.is_valid(entity));

        if !self.has_component::<C>(entity) {
            return None;
        }

        match self.component_datastructures.get(&TypeId::of::<C>().hash()) {
            Some(&ComponentDatastructure::VecMap) => {
                // TODO unsafe unwrap here, because we know this entry exists
                let component_list = self.component_lists.get::<VecMap<C>>().unwrap();
                component_list.get(&entity.index())
            },
            Some(&ComponentDatastructure::HashMap) => {
                // TODO unsafe unwrap here, because we know this entry exists
                let component_list = self.component_lists.get::<HashMap<uint, C>>().unwrap();
                component_list.get(&entity.index())
            },
            None => panic!("Tried to assign unregistered component"),
        }
    }

    pub fn get_component_mut<C: 'static>(&'a mut self, entity: &Entity<Id>) -> Option<&mut C> {
        assert!(self.is_valid(entity));

        if !self.has_component::<C>(entity) {
            return None;
        }

        match self.component_datastructures.get(&TypeId::of::<C>().hash()) {
            Some(&ComponentDatastructure::VecMap) => {
                // TODO unsafe unwrap here, because we know this entry exists
                let component_list = self.component_lists.get_mut::<VecMap<C>>().unwrap();
                component_list.get_mut(&entity.index())
            },
            Some(&ComponentDatastructure::HashMap) => {
                // TODO unsafe unwrap here, because we know this entry exists
                let component_list = self.component_lists.get_mut::<HashMap<uint, C>>().unwrap();
                component_list.get_mut(&entity.index())
            },
            None => panic!("Tried to assign unregistered component"),
        }
    }

    pub fn entities(&self) -> EntityIterator<Id> {
        EntityIterator {
            entity_manager: self,
            last_entity_index: self.next_entity_index - 1, // last valid entity index
            index: 0,
            free_entity_index_list: self.free_entity_index_list.iter(),
        }
    }
}

impl<Id> PartialEq for EntityManager<Id> {
    fn eq(&self, other: &EntityManager<Id>) -> bool {
        self == other
    }
}

pub struct EntityIterator<'a, Id: 'a> {
    entity_manager: &'a EntityManager<Id>,
    last_entity_index: uint,
    index: uint,
    free_entity_index_list: binary_heap::Items<'a, uint>,
}

impl<'a, Id> Iterator<Entity<Id>> for EntityIterator<'a, Id> {
    fn next(&mut self) -> Option<Entity<Id>> {
        // for all valid entity indexes
        while self.index <= self.last_entity_index {
            let mut free_entity_index = -1;

            // find if the index is in the free_entity_index_list
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

            let version = self.entity_manager.entity_versions[self.index];

            let result = Some(Entity {
                id: EntityId {
                    index: self.index,
                    version: version,
                },
                marker: self.entity_manager.marker,
            });

            self.index += 1;
            return result;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use entity::{ EntityManager, ComponentDatastructure };

    #[test]
    fn created_entity_is_valid() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        let entity = entity_manager.create_entity();
        assert!(entity_manager.is_valid(&entity));
    }

    #[test]
    fn deleted_entity_is_invalid() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        let entity1 = entity_manager.create_entity();
        let entity1_copy = entity1;

        assert!(entity_manager.is_valid(&entity1_copy));
        entity_manager.destroy_entity(entity1);
        assert!(!entity_manager.is_valid(&entity1_copy));
    }

    #[test]
    fn create_reuses_index() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        let entity1 = entity_manager.create_entity();
        let entity1_copy = entity1;

        entity_manager.destroy_entity(entity1);

        let entity3 = entity_manager.create_entity();
        assert_eq!(entity3.id.index, entity1_copy.id.index);
        assert_eq!(entity3.id.version, entity1_copy.id.version + 1);
    }

    #[test]
    fn components() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        // test different datastructures
        #[deriving(PartialEq, Show)]
        struct UnitComponent;
        entity_manager.register_component::<UnitComponent>(ComponentDatastructure::VecMap);

        #[deriving(PartialEq, Show)]
        struct TupleComponent(int);
        entity_manager.register_component::<TupleComponent>(ComponentDatastructure::HashMap);

        #[deriving(PartialEq, Show)]
        struct Component {
            field: int,
        }
        entity_manager.register_component::<Component>(ComponentDatastructure::VecMap);

        let entity = entity_manager.create_entity();

        // test unassigned components are None
        {
            let unit_component = entity_manager.get_component::<UnitComponent>(&entity);
            assert!(unit_component.is_none());

            let tuple_component = entity_manager.get_component::<TupleComponent>(&entity);
            assert!(tuple_component.is_none());

            let component = entity_manager.get_component::<Component>(&entity);
            assert!(component.is_none());
        }

        entity_manager.assign_component::<UnitComponent>(&entity, UnitComponent);
        entity_manager.assign_component::<TupleComponent>(&entity, TupleComponent(1));
        entity_manager.assign_component::<Component>(&entity, Component { field: 1 });

        // test assigned components
        {
            let unit_component = entity_manager.get_component::<UnitComponent>(&entity);
            assert_eq!(unit_component.unwrap(), &UnitComponent);

            let tuple_component = entity_manager.get_component::<TupleComponent>(&entity);
            assert_eq!(tuple_component.unwrap(), &TupleComponent(1));

            let component = entity_manager.get_component::<Component>(&entity);
            assert_eq!(component.unwrap(), &Component { field: 1 });
        }
    }

    #[test]
    #[should_fail]
    fn register_component_twice() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        struct Component;
        entity_manager.register_component::<Component>(ComponentDatastructure::VecMap);
        entity_manager.register_component::<Component>(ComponentDatastructure::VecMap);
    }
}