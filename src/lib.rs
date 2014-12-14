#![macro_escape]
#![feature(macro_rules)]
extern crate anymap;

use anymap::AnyMap;

use std::rc::{ Rc, Weak };
use std::cell::RefCell;
use std::iter::{ Iterator };
use std::collections::{ BinaryHeap, Bitv, VecMap, HashMap };
use std::uint;
use std::intrinsics::TypeId;
use std::fmt::Show;

pub use tup_append::TupAppend;

pub mod tup_append;

pub trait EntityBuilder: 'static
{
    fn build(&mut self, &mut EntityManager, Entity);
}

impl EntityBuilder for |&mut EntityManager, Entity|: 'static
{
    fn build(&mut self, c: &mut EntityManager, e: Entity)
    {
        (*self)(c, e);
    }
}

pub trait EntityModifier: 'static
{
    fn modify(&mut self, &mut EntityManager, Entity);
}

impl EntityModifier for |&mut EntityManager, Entity|: 'static
{
    fn modify(&mut self, c: &mut EntityManager, e: Entity)
    {
        (*self)(c, e);
    }
}

pub struct Control<'a> {
    builders: Vec<Box<EntityBuilder + 'static>>,
    destroyed: Vec<Entity>,
    modifiers: Vec<(Entity, Box<EntityModifier + 'static>)>,
}

impl<'a> Control<'a> {
    fn new() -> Control<'a> {
        Control {
            builders: Vec::new(),
            destroyed: Vec::new(),
            modifiers: Vec::new(),
        }
    }

    pub fn build(&mut self, builder: Box<EntityBuilder + 'static>) {
        self.builders.push(builder);
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.destroyed.push(entity);
    }

    pub fn modify(&mut self, entity: Entity, modifier: Box<EntityModifier + 'static>) {
        self.modifiers.push((entity, modifier));
    }

    fn apply(self, entity_manager: &Rc<RefCell<EntityManager>>) {
        let mut entity_manager = entity_manager.borrow_mut();
        for mut builder in self.builders.into_iter() {
            let entity = entity_manager.deref_mut().create_entity();
            builder.build(entity_manager.deref_mut(), entity);
        }

        for (entity, mut modifier) in self.modifiers.into_iter() {
            modifier.modify(entity_manager.deref_mut(), entity);
        }

        for entity in self.destroyed.into_iter() {
            entity_manager.deref_mut().destroy_entity(entity);
        }
    }
}

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
    free_entity_index_list: std::collections::binary_heap::Items<'a, uint>,
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
            if let &Some(_) = $em.get_component::<$ty>(&tuple.0) {
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
)

#[macro_export]
macro_rules! entities_with_components(
    ( $em:ident : $( $kinds:ident $types:path )* ) => (
        $em.entities().filter_map(|entity|
            entities_with_components_inner!($em, Some((entity,)): $( $kinds $types )* )
        )
    );
)
