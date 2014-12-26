use std::collections::{ Bitv };

use entity::{ EntityManager, Entity, EntityIterator };
use entity::{ ComponentList, ComponentData };
use system::{ SystemManager, System };

pub struct World<WorldId> {
    entity_manager: EntityManager<WorldId>,
    system_manager: SystemManager<WorldId>,
}

impl<'a, WorldId> World<WorldId> {
    pub fn new() -> World<WorldId> {
        World {
            entity_manager: EntityManager::new(),
            system_manager: SystemManager::new(),
        }
    }

    // *** EntityManager ***

    pub fn create_entity(&mut self) -> Entity<WorldId> {
        self.entity_manager.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity<WorldId>) {
        self.entity_manager.destroy_entity(entity)
    }

    pub fn is_valid(&self, entity: &Entity<WorldId>) -> bool {
        self.entity_manager.is_valid(entity)
    }

    pub fn entities(&self) -> EntityIterator<WorldId> {
        self.entity_manager.entities()
    }

    // *** ComponentManager ***

    pub fn register_component<C: 'static>(&mut self, component_list: Box<ComponentList<'a, C> + 'static>) {
        self.entity_manager.register_component(component_list)
    }

    pub fn assign_component<C: 'static>(&mut self, entity: &Entity<WorldId>, component: C) {
        self.entity_manager.assign_component(entity, component)
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity<WorldId>) -> bool {
        self.entity_manager.has_component::<C>(entity)
    }

    pub fn get_component<C: 'static>(&'a self, entity: &Entity<WorldId>) -> Option<&C> {
        self.entity_manager.get_component::<C>(entity)
    }

    pub fn get_component_mut<C: 'static>(&'a mut self, entity: &Entity<WorldId>) -> Option<&mut C> {
        self.entity_manager.get_component_mut::<C>(entity)
    }

    pub fn get_component_data<C: 'static>(&'a self) -> &ComponentData<C> {
        self.entity_manager.get_component_data::<C>()
    }

    pub fn get_component_data_mut<C: 'static>(&'a mut self) -> &mut ComponentData<C> {
        self.entity_manager.get_component_data_mut::<C>()
    }

    pub fn get_entity_component_mask(&self, entity: &Entity<WorldId>) -> &Bitv {
        self.entity_manager.get_entity_component_mask(entity)
    }

    pub fn get_components_length(&self) -> uint {
        self.entity_manager.get_components_length()
    }

    // *** SystemManager ***

    pub fn register_system<S>(&mut self, system: S) where S: System<WorldId, S> + 'static {
        self.system_manager.register(system)
    }

    pub fn update_system<A, S>(&mut self, args: &A) where S: System<WorldId, S> + 'static {
        self.system_manager.update::<A,S>(&mut self.entity_manager, args)
    }
}