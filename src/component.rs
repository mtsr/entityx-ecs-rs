use std::collections::{ Bitv, VecMap, HashMap };

use anymap::AnyMap;

use entity::{ Entity };

// TODO more DB like approach to ECS i.e. more powerful query tools
// TODO optimize getting entities with components
// by starting from the 'with' component with the fewest instances
// TODO Add Component Copy-on-Write from Template

pub struct ComponentData<'a, Component: 'static> {
    pub index: uint,
    pub list: Box<ComponentList<'a, Component> + 'static>
    // TODO consider adding a counter to allow ordering component checking
    // when iterating over entities with components
}

// TODO Add BTreeMap
pub trait ComponentList<'a, Component> {
    fn contains_key(&self, &uint) -> bool;
    fn get(&self, &uint) -> Option<&Component>;
    fn get_mut(&mut self, &uint) -> Option<&mut Component>;
    fn insert(&mut self, uint, Component);
    fn remove(&mut self, key: &uint) -> Option<Component>;
    // TODO figure out return type
    // fn iter(&'a self) -> Iterator<(uint, &'a Component)>;
    // fn iter_mut(&'a self) -> Iterator<(uint, &'a mut Component)>;
}

impl<'a, Component> ComponentList<'a, Component> for VecMap<Component> {
    fn contains_key(&self, index: &uint) -> bool { self.contains_key(index) }
    fn get(&self, index: &uint) -> Option<&Component> { self.get(index) }
    fn get_mut(&mut self, index: &uint) -> Option<&mut Component> { self.get_mut(index) }
    fn insert(&mut self, index: uint, component: Component) { self.insert(index, component); }
    fn remove(&mut self, key: &uint) -> Option<Component> { self.remove(key) }
    // TODO figure out return type
    // fn iter(&'a self) -> Iterator<(uint, &'a Component)> { self.iter() }
    // fn iter_mut(&'a self) -> Iterator<(uint, &'a mut Component)> { self.iter_mut() }
}

impl<'a, Component> ComponentList<'a, Component> for HashMap<uint, Component> {
    fn contains_key(&self, index: &uint) -> bool { self.contains_key(index) }
    fn get(&self, index: &uint) -> Option<&Component> { self.get(index) }
    fn get_mut(&mut self, index: &uint) -> Option<&mut Component> { self.get_mut(index) }
    fn insert(&mut self, index: uint, component: Component) { self.insert(index, component); }
    fn remove(&mut self, key: &uint) -> Option<Component> { self.remove(key) }
    // TODO figure out return type
    // fn iter(&'a self) -> Iterator<(uint, &'a Component)> { self.iter() }
    // fn iter_mut(&'a self) -> Iterator<(uint, &'a mut Component)> { self.iter_mut() }
}

pub struct ComponentManager<WorldId> {
    entity_component_masks: Vec<Bitv>,
    next_component_index: uint,
    component_data: AnyMap,
}

impl<'a, WorldId> ComponentManager<WorldId> {
    pub fn new(initial_capacity: uint) -> ComponentManager<WorldId> {
        ComponentManager {
            entity_component_masks: Vec::with_capacity(initial_capacity),
            next_component_index: 0,
            component_data: AnyMap::new(),
        }
    }

    pub fn entity_created(&mut self, entity: &Entity<WorldId>) {
        // Assumes entities are always created using continuous indices
        if self.entity_component_masks.len() < entity.index() {
            panic!("Entity with non-continuous index created!");
        } else if self.entity_component_masks.len() == entity.index() {
            self.entity_component_masks.push(Bitv::from_elem(self.next_component_index, false));
        }
    }

    pub fn entity_destroyed(&mut self, entity: &Entity<WorldId>) {
        self.entity_component_masks[entity.index()].clear();
    }

    pub fn register_component<C: 'static>(&mut self, component_list: Box<ComponentList<'a, C> + 'static>) {
        match self.component_data.get::<ComponentData<C>>() {
            None => {
                self.component_data.insert::<ComponentData<C>>(ComponentData {
                    index: self.next_component_index,
                    list: component_list,
                });

                self.next_component_index += 1;

                for mut entity_component_mask in self.entity_component_masks.iter_mut() {
                    // dynamically grow bitv length, only needed if new component types can be registered after entities have been added
                    entity_component_mask.grow(self.next_component_index, false);
                }
            },
            Some(_) => panic!("Tried to register component twice"),
        }
    }

    /// Add or replace component on entity
    pub fn assign_component<C: 'static>(&mut self, entity: &Entity<WorldId>, component: C) {
        let index = {
            let component_data = self.get_component_data_mut::<C>();
            component_data.list.insert(entity.index(), component);
            component_data.index
        };

        self.entity_component_masks[entity.index()].set(index, true);
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity<WorldId>) -> bool {
        let component_data = self.get_component_data::<C>();
        self.entity_component_masks[entity.index()].get(component_data.index).unwrap()
    }

    // TODO dedup get_component and get_component_mut
    pub fn get_component<C: 'static>(&'a self, entity: &Entity<WorldId>) -> Option<&C> {
        let component_data = self.get_component_data::<C>();

        let has_component = self.entity_component_masks[entity.index()].get(component_data.index).unwrap();
        if !has_component {
            return None;
        }

        component_data.list.get(&entity.index())
    }

    pub fn get_component_mut<C: 'static>(&'a mut self, entity: &Entity<WorldId>) -> Option<&mut C> {
        // TODO get rid of double get_component_data + get_component_data_mut
        if !self.has_component::<C>(entity) {
            return None;
        }

        let component_data = self.get_component_data_mut::<C>();
        component_data.list.get_mut(&entity.index())
    }

    pub fn get_component_data<C: 'static>(&'a self) -> &ComponentData<C> {
        if let Some(component_data) = self.component_data.get::<ComponentData<C>>() {
            component_data
        } else {
            panic!("Tried to get unregistered component");
        }
    }

    pub fn get_component_data_mut<C: 'static>(&'a mut self) -> &mut ComponentData<C> {
        if let Some(component_data) = self.component_data.get_mut::<ComponentData<C>>() {
            component_data
        } else {
            panic!("Tried to get unregistered component");
        }
    }

    pub fn get_entity_component_mask(&self, entity: &Entity<WorldId>) -> &Bitv {
        &self.entity_component_masks[entity.index()]
    }

    pub fn get_components_length(&self) -> uint {
        self.next_component_index
    }

}

#[cfg(test)]
mod tests {
    use super::{
        ComponentManager
    };
    use entity::{ EntityManager };
    use std::collections::{ VecMap, HashMap };

    #[test]
    fn register_components() {
        struct WorldId1;
        let mut component_manager: ComponentManager<WorldId1> = ComponentManager::new(256);

        // test different datastructures
        #[deriving(PartialEq, Show)]
        struct UnitComponent;
        component_manager.register_component::<UnitComponent>(box VecMap::new());

        #[deriving(PartialEq, Show)]
        struct TupleComponent(int);
        component_manager.register_component::<TupleComponent>(box HashMap::new());

        #[deriving(PartialEq, Show)]
        struct Component {
            field: int,
        }
        component_manager.register_component::<Component>(box VecMap::new());
    }

    #[test]
    fn unassigned_components() {
        struct WorldId1;
        let mut component_manager: ComponentManager<WorldId1> = ComponentManager::new(256);
        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        // test different datastructures
        #[deriving(PartialEq, Show)]
        struct UnitComponent;
        component_manager.register_component::<UnitComponent>(box VecMap::new());

        #[deriving(PartialEq, Show)]
        struct TupleComponent(int);
        component_manager.register_component::<TupleComponent>(box HashMap::new());

        #[deriving(PartialEq, Show)]
        struct Component {
            field: int,
        }
        component_manager.register_component::<Component>(box VecMap::new());

        let entity = entity_manager.create_entity();
        component_manager.entity_created(&entity);

        // test unassigned components are None
        let unit_component = component_manager.get_component::<UnitComponent>(&entity);
        assert!(unit_component.is_none());

        let tuple_component = component_manager.get_component::<TupleComponent>(&entity);
        assert!(tuple_component.is_none());

        let component = component_manager.get_component::<Component>(&entity);
        assert!(component.is_none());
    }

    #[test]
    fn assigned_components() {
        struct WorldId1;
        let mut component_manager: ComponentManager<WorldId1> = ComponentManager::new(256);
        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        // test different datastructures
        #[deriving(PartialEq, Show)]
        struct UnitComponent;
        component_manager.register_component::<UnitComponent>(box VecMap::new());

        #[deriving(PartialEq, Show)]
        struct TupleComponent(int);
        component_manager.register_component::<TupleComponent>(box HashMap::new());

        #[deriving(PartialEq, Show)]
        struct Component {
            field: int,
        }
        component_manager.register_component::<Component>(box VecMap::new());

        let entity = entity_manager.create_entity();
        component_manager.entity_created(&entity);

        component_manager.assign_component::<UnitComponent>(&entity, UnitComponent);
        component_manager.assign_component::<TupleComponent>(&entity, TupleComponent(1));
        component_manager.assign_component::<Component>(&entity, Component { field: 1 });

        // test assigned components
        let unit_component = component_manager.get_component::<UnitComponent>(&entity);
        assert_eq!(unit_component.unwrap(), &UnitComponent);

        let tuple_component = component_manager.get_component::<TupleComponent>(&entity);
        assert_eq!(tuple_component.unwrap(), &TupleComponent(1));

        let component = component_manager.get_component::<Component>(&entity);
        assert_eq!(component.unwrap(), &Component { field: 1 });
    }

    #[test]
    #[should_fail]
    fn register_component_twice() {
        struct WorldId1;
        let mut component_manager: ComponentManager<WorldId1> = ComponentManager::new(256);

        struct Component;
        component_manager.register_component::<Component>(box VecMap::new());
        component_manager.register_component::<Component>(box VecMap::new());
    }
}