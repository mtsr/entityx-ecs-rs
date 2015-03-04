use std::marker::PhantomData;
use std::collections::{ BitVec, VecMap, HashMap };

use anymap::AnyMap;

use entity::{ Entity };

// TODO more DB like approach to ECS i.e. more powerful query tools
// TODO Add Component Copy-on-Write from Template
// TODO Consider using unsafe for transmuting Option
// use std::mem::transmute;

pub struct ComponentData<Component: 'static> {
    pub index: usize,
    pub list: Box<ComponentList<Component> + 'static>
}

// TODO Add BTreeMap
pub trait ComponentList<Component> {
    fn contains_key(&self, &usize) -> bool;
    fn get(&self, &usize) -> Option<&Component>;
    fn get_mut(&mut self, &usize) -> Option<&mut Component>;
    fn insert(&mut self, usize, Component);
    fn remove(&mut self, key: &usize) -> Option<Component>;
    // fn iter(&self) -> Box<Iterator<Item=(usize, &Component)>>;
    // fn iter_mut(&mut self) -> Box<Iterator<Item=(usize, &mut Component)>>;
}

impl<Component> ComponentList<Component> for VecMap<Component> {
    fn contains_key(&self, index: &usize) -> bool { self.contains_key(index) }
    fn get(&self, index: &usize) -> Option<&Component> { self.get(index) }
    fn get_mut(&mut self, index: &usize) -> Option<&mut Component> { self.get_mut(index) }
    fn insert(&mut self, index: usize, component: Component) { self.insert(index, component); }
    fn remove(&mut self, key: &usize) -> Option<Component> { self.remove(key) }
    // fn iter(&self) -> Box<Iterator<Item=(usize, &Component)>> { Box::new(self.iter()) }
    // fn iter_mut(&mut self) -> Box<Iterator<Item=(usize, &mut Component)>> { Box::new(self.iter_mut()) }
}

impl<Component> ComponentList<Component> for HashMap<usize, Component> {
    fn contains_key(&self, index: &usize) -> bool { self.contains_key(index) }
    fn get(&self, index: &usize) -> Option<&Component> { self.get(index) }
    fn get_mut(&mut self, index: &usize) -> Option<&mut Component> { self.get_mut(index) }
    fn insert(&mut self, index: usize, component: Component) { self.insert(index, component); }
    fn remove(&mut self, key: &usize) -> Option<Component> { self.remove(key) }
    // fn iter(&self) -> Box<Iterator<Item=(usize, &Component)>> { Box::new(self.iter().map(|(index, component)| (*index, component))) }
    // fn iter_mut(&mut self) -> Box<Iterator<Item=(usize, &mut Component)>> { Box::new(self.iter_mut().map(|(index, component)| (*index, component))) }
}

pub struct ComponentManager<WorldId> {
    phantom: PhantomData<WorldId>,
    entity_component_masks: Vec<BitVec>,
    next_component_index: usize,
    component_data: AnyMap,
}

impl<'a, WorldId> ComponentManager<WorldId> {
    pub fn new(initial_capacity: usize) -> ComponentManager<WorldId> {
        ComponentManager {
            phantom: PhantomData,
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
            self.entity_component_masks.push(BitVec::from_elem(self.next_component_index, false));
        }
    }

    pub fn entity_destroyed(&mut self, entity: &Entity<WorldId>) {
        self.entity_component_masks[entity.index()].clear();
    }

    pub fn register_component<C: 'static>(&mut self, component_list: Box<ComponentList<C> + 'static>) {
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

    pub fn remove_component<C: 'static>(&mut self, entity: &Entity<WorldId>) {
        let index = {
            let component_data = self.get_component_data_mut::<C>();
            component_data.list.remove(&entity.index());
            component_data.index
        };

        self.entity_component_masks[entity.index()].set(index, false);
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity<WorldId>) -> bool {
        let component_data = self.get_component_data::<C>();
        self.has_component_from_data::<C>(entity, component_data)
    }

    #[inline]
    fn has_component_from_data<C: 'static>(&self, entity: &Entity<WorldId>, component_data: &ComponentData<C>) -> bool {
        self.entity_component_masks[entity.index()].get(component_data.index).unwrap()
    }

    pub fn get_component<C: 'static>(&'a self, entity: &Entity<WorldId>) -> Option<&C> {
        let component_data = self.get_component_data::<C>();
        let component = component_data.list.get(&entity.index());
        if component.is_some() && !self.has_component_from_data(entity, component_data) {
            None
        } else {
            component
        }
    }

    pub fn get_component_mut<C: 'static>(&'a mut self, entity: &Entity<WorldId>) -> Option<&mut C> {
        // TODO figure out how to do this without cloning
        let entity_component_mask = self.entity_component_masks[entity.index()].clone();

        let component_data = self.get_component_data_mut::<C>();
        let component = component_data.list.get_mut(&entity.index());
        if component.is_some() && !entity_component_mask.get(component_data.index).unwrap() {
            None
        } else {
            component
        }
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

    pub fn get_entity_component_mask(&self, entity: &Entity<WorldId>) -> &BitVec {
        &self.entity_component_masks[entity.index()]
    }

    pub fn get_components_length(&self) -> usize {
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
        #[derive(PartialEq, Debug)]
        struct UnitComponent;
        component_manager.register_component::<UnitComponent>(Box::new(VecMap::new()));

        #[derive(PartialEq, Debug)]
        struct TupleComponent(isize);
        component_manager.register_component::<TupleComponent>(Box::new(HashMap::new()));

        #[derive(PartialEq, Debug)]
        struct Component {
            field: isize,
        }
        component_manager.register_component::<Component>(Box::new(VecMap::new()));
    }

    #[test]
    fn unassigned_components() {
        struct WorldId1;
        let mut component_manager: ComponentManager<WorldId1> = ComponentManager::new(256);
        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        // test different datastructures
        #[derive(PartialEq, Debug)]
        struct UnitComponent;
        component_manager.register_component::<UnitComponent>(Box::new(VecMap::new()));

        #[derive(PartialEq, Debug)]
        struct TupleComponent(isize);
        component_manager.register_component::<TupleComponent>(Box::new(HashMap::new()));

        #[derive(PartialEq, Debug)]
        struct Component {
            field: isize,
        }
        component_manager.register_component::<Component>(Box::new(VecMap::new()));

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
        #[derive(PartialEq, Debug)]
        struct UnitComponent;
        component_manager.register_component::<UnitComponent>(Box::new(VecMap::new()));

        #[derive(PartialEq, Debug)]
        struct TupleComponent(isize);
        component_manager.register_component::<TupleComponent>(Box::new(HashMap::new()));

        #[derive(PartialEq, Debug)]
        struct Component {
            field: isize,
        }
        component_manager.register_component::<Component>(Box::new(VecMap::new()));

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
        component_manager.register_component::<Component>(Box::new(VecMap::new()));
        component_manager.register_component::<Component>(Box::new(VecMap::new()));
    }
}