use std::marker::PhantomData;

use entity::{ EntityManager, Entity };

pub trait EntityBuilder<WorldId, S>: 'static {
    fn build(&mut self, &mut EntityManager<WorldId>, &mut S, Entity<WorldId>);
}

impl<WorldId, S> EntityBuilder<WorldId, S> for Fn(&mut EntityManager<WorldId>, &mut S, Entity<WorldId>) + 'static {
    fn build(&mut self, entity_manager: &mut EntityManager<WorldId>, system: &mut S, entity: Entity<WorldId>) {
        (*self)(entity_manager, system, entity);
    }
}

pub trait EntityModifier<WorldId, S>: 'static {
    fn modify(&mut self, &mut EntityManager<WorldId>, &mut S, Entity<WorldId>);
}

impl<WorldId, S> EntityModifier<WorldId, S> for Fn(&mut EntityManager<WorldId>, &mut S, Entity<WorldId>) + 'static {
    fn modify(&mut self, entity_manager: &mut EntityManager<WorldId>, system: &mut S, entity: Entity<WorldId>) {
        (*self)(entity_manager, system, entity);
    }
}

pub struct Control<WorldId, S> {
    phantom: PhantomData<WorldId>,
    builders: Vec<Box<EntityBuilder<WorldId, S> + 'static>>,
    destroyed: Vec<Entity<WorldId>>,
    modifiers: Vec<(Entity<WorldId>, Box<EntityModifier<WorldId, S> + 'static>)>,
}

impl<WorldId, S> Control<WorldId, S> {
    pub fn new() -> Control<WorldId, S> {
        Control {
            phantom: PhantomData,
            builders: Vec::new(),
            destroyed: Vec::new(),
            modifiers: Vec::new(),
        }
    }

    pub fn build(&mut self, builder: Box<EntityBuilder<WorldId, S> + 'static>) {
        self.builders.push(builder);
    }

    pub fn destroy(&mut self, entity: Entity<WorldId>) {
        self.destroyed.push(entity);
    }

    pub fn modify(&mut self, entity: Entity<WorldId>, modifier: Box<EntityModifier<WorldId, S> + 'static>) {
        self.modifiers.push((entity, modifier));
    }

    pub fn apply(self, entity_manager: &mut EntityManager<WorldId>, system: &mut S) {
        let mut entity_manager = entity_manager;
        for mut builder in self.builders.into_iter() {
            let entity = entity_manager.create_entity();
            builder.build(entity_manager, system, entity);
        }

        for (entity, mut modifier) in self.modifiers.into_iter() {
            modifier.modify(entity_manager, system, entity);
        }

        for entity in self.destroyed.into_iter() {
            entity_manager.destroy_entity(entity);
        }
    }
}