#![macro_escape]

use std::collections::{ BinaryHeap, VecMap, HashMap };
use std::collections::binary_heap::{ Iter };

// TODO Consider using unsafe for transmuting Option
// use std::mem::transmute;

// TODO Add Entity Templates
// TODO Add Component Copy-on-Write from Template

use std::iter::{ Iterator };
use std::uint;
use std::fmt::{ Show, Formatter, Error };

use std::kinds::marker;

// TODO more DB like approach to ECS i.e. more powerful query tools
// TODO optimize getting entities with components
// by starting from the 'with' component with the fewest instances

use anymap::AnyMap;

#[deriving(Show,Clone)]
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
    pub fn id(&self) -> &EntityId {
        &self.id
    }
}

impl<Id> Clone for Entity<Id> {
    fn clone(&self) -> Self {
        Entity {
            id: self.id.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<Id> Show for Entity<Id> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        self.id.fmt(formatter)
    }
}

impl<Id> PartialEq for Entity<Id> {
    fn eq(&self, other: &Entity<Id>) -> bool {
        self.id() == other.id()
    }
}

pub type ComponentId = u64;

struct ComponentData<'a, Component: 'static> {
    list: Box<ComponentList<'a, Component> + 'static>
}

pub trait ComponentList<'a, Component> {
    fn contains_key(&self, &uint) -> bool;
    fn get(&self, &uint) -> Option<&Component>;
    fn get_mut(&mut self, &uint) -> Option<&mut Component>;
    fn insert(&mut self, uint, Component);
    // TODO figure out return type
    // fn iter(&'a self) -> Iterator<(uint, &'a Component)>;
    // fn iter_mut(&'a self) -> Iterator<(uint, &'a mut Component)>;
}

impl<'a, Component> ComponentList<'a, Component> for VecMap<Component> {
    fn contains_key(&self, index: &uint) -> bool { self.contains_key(index) }
    fn get(&self, index: &uint) -> Option<&Component> { self.get(index) }
    fn get_mut(&mut self, index: &uint) -> Option<&mut Component> { self.get_mut(index) }
    fn insert(&mut self, index: uint, component: Component) { self.insert(index, component); }
    // TODO figure out return type
    // fn iter(&'a self) -> Iterator<(uint, &'a Component)> { self.iter() }
    // fn iter_mut(&'a self) -> Iterator<(uint, &'a mut Component)> { self.iter_mut() }
}

impl<'a, Component> ComponentList<'a, Component> for HashMap<uint, Component> {
    fn contains_key(&self, index: &uint) -> bool { self.contains_key(index) }
    fn get(&self, index: &uint) -> Option<&Component> { self.get(index) }
    fn get_mut(&mut self, index: &uint) -> Option<&mut Component> { self.get_mut(index) }
    fn insert(&mut self, index: uint, component: Component) { self.insert(index, component); }
    // TODO figure out return type
    // fn iter(&'a self) -> Iterator<(uint, &'a Component)> { self.iter() }
    // fn iter_mut(&'a self) -> Iterator<(uint, &'a mut Component)> { self.iter_mut() }
}

pub struct EntityManager<Id> {
    marker: marker::InvariantType<Id>,

    next_entity_index: uint,
    free_entity_index_list: BinaryHeap<uint>,

    entity_versions: Vec<uint>,

    // TODO replace with HashMap<TypeId, Any>
    // Where Any is Vec<Option<C>> VecMap<Option<C>> or HashMap<Option<C>>
    // so that it's possible to access component lists without <C>
    // TODO Add BTreeMap
    component_data: AnyMap,
}

impl<'a, Id> EntityManager<Id> {
    pub fn new() -> EntityManager<Id> {
        EntityManager {
            marker: marker::InvariantType,

            next_entity_index: 0,
            free_entity_index_list: BinaryHeap::with_capacity(32),

            entity_versions: Vec::from_elem(256, 0u),

            component_data: AnyMap::new(),
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

        if index >= self.entity_versions.capacity() {
            // grow increases capacity in a smart way
            // no reason to specify particular size here
            self.entity_versions.grow(index, 0u);
        }

        let version = self.entity_versions[index];
        EntityId {
            index: index,
            version: version,
        }
    }

    pub fn create_entity(&mut self) -> Entity<Id> {
        Entity {
            id: self.create_id(),
            marker: self.marker,
        }
    }

    pub fn destroy_entity(&mut self, entity: Entity<Id>) {
        // TODO clear/invalidate component data
        self.entity_versions[entity.index()] += 1;
        self.free_entity_index_list.push(entity.index());
    }

    pub fn is_valid(&self, entity: &Entity<Id>) -> bool {
        entity.index() < self.next_entity_index
        && entity.version() == self.entity_versions[entity.index()]
    }

    // TODO look into moving datastructure type into type parameter
    pub fn register_component<C: 'static>(&mut self, component_list: Box<ComponentList<'a, C> + 'static>) {
        if let None = self.component_data.get::<ComponentData<C>>() {
            self.component_data.insert::<ComponentData<C>>(ComponentData {
                list: component_list,
            });
        } else {
            panic!("Tried to register component twice");
        }
    }

    /// Add or replace component on entity
    pub fn assign_component<C: 'static>(&mut self, entity: &Entity<Id>, component: C) {
        assert!(self.is_valid(entity));

        if let Some(mut component_data) = self.component_data.get_mut::<ComponentData<C>>() {
            component_data.list.insert(entity.index(), component);
        } else {
            panic!("Tried to assign unregistered component");
        }
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity<Id>) -> bool {
        assert!(self.is_valid(entity));

        if let Some(component_data) = self.component_data.get::<ComponentData<C>>() {
            component_data.list.contains_key(&entity.index())
        } else {
            panic!("Tried to check for unregistered component");
        }
    }

    // TODO dedup get_component and get_component_mut
    pub fn get_component<C: 'static>(&'a self, entity: &Entity<Id>) -> Option<&C> {
        assert!(self.is_valid(entity));

        if let Some(component_data) = self.component_data.get::<ComponentData<C>>() {
            component_data.list.get(&entity.index())
        } else {
            panic!("Tried to get unregistered component");
        }
    }

    pub fn get_component_mut<C: 'static>(&'a mut self, entity: &Entity<Id>) -> Option<&mut C> {
        assert!(self.is_valid(entity));

        if let Some(mut component_data) = self.component_data.get_mut::<ComponentData<C>>() {
            component_data.list.get_mut(&entity.index())
        } else {
            panic!("Tried to get unregistered component");
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
    free_entity_index_list: Iter<'a, uint>,
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

        None::<Entity<Id>>
    }
}

// TODO allow with Player(1) style queries.

#[macro_export]
macro_rules! entities_with_components_inner(
    ( $em:ident, $already:expr : ) => ( $already );
    ( $em:ident, $already:expr : with $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $em, $already.and_then(|tuple| {
            let comp = $em.get_component::<$ty>(&tuple.0);
            match comp {
                Some(obj) => Some( tuple.tup_append(obj) ),
                None => None
            }
        } ) : $( $kinds $types )* )
    );
    ( $em:ident, $already:expr : without $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $em, $already.and_then(|tuple|
            if let Some(_) = $em.get_component::<$ty>(&tuple.0) {
                None
            } else {
                Some(tuple)
            }
        ) : $( $kinds $types )* )
    );
    ( $em:ident, $already:expr : option $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $em, $already.map(|tuple| {
            let comp = $em.get_component::<$ty>(&tuple.0);
            tuple.tup_append( comp )
        } ) : $( $kinds $types )* )
    );
);

#[macro_export]
macro_rules! entities_with_components(
    ( $em:ident : $( $kinds:ident $types:path )* ) => (
        $em.entities().filter_map(|entity|
            entities_with_components_inner!($em, Some((entity,)): $( $kinds $types )* )
        )
    );
);

#[cfg(test)]
mod tests {
    use entity::{ EntityManager };
    use std::collections::{ VecMap, HashMap };
    use tup_append::TupAppend;

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
        let entity1_clone = entity1.clone();

        assert!(entity_manager.is_valid(&entity1_clone));
        entity_manager.destroy_entity(entity1);
        assert!(!entity_manager.is_valid(&entity1_clone));
    }

    #[test]
    fn create_reuses_index() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        let entity1 = entity_manager.create_entity();
        let entity1_clone = entity1.clone();

        entity_manager.destroy_entity(entity1);

        let entity3 = entity_manager.create_entity();
        assert_eq!(entity3.id.index, entity1_clone.id.index);
        assert_eq!(entity3.id.version, entity1_clone.id.version + 1);
    }

    #[test]
    fn components() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        // test different datastructures
        #[deriving(PartialEq, Show)]
        struct UnitComponent;
        entity_manager.register_component::<UnitComponent>(box VecMap::new());

        #[deriving(PartialEq, Show)]
        struct TupleComponent(int);
        entity_manager.register_component::<TupleComponent>(box HashMap::new());

        #[deriving(PartialEq, Show)]
        struct Component {
            field: int,
        }
        entity_manager.register_component::<Component>(box VecMap::new());

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
        entity_manager.register_component::<Component>(box VecMap::new());
        entity_manager.register_component::<Component>(box VecMap::new());
    }

    #[test]
    fn macro() {
        struct World1;
        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        #[deriving(PartialEq,Show)]
        struct Component;

        entity_manager.register_component::<Component>(box VecMap::new());

        let entity = entity_manager.create_entity();
        entity_manager.assign_component::<Component>(&entity, Component);

        for (_, component) in entities_with_components!(entity_manager: with Component) {
            assert_eq!(component, &Component);
        }
    }
}