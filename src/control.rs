use std::rc::{ Rc };
use std::cell::RefCell;

use entity::{ EntityManager, Entity };

pub trait EntityBuilder<Id, S>: 'static {
    fn build(&mut self, &mut EntityManager<Id>, &mut S, Entity<Id>);
}

impl<Id, S> EntityBuilder<Id, S> for |&mut EntityManager<Id>, &mut S, Entity<Id>|: 'static {
    fn build(&mut self, c: &mut EntityManager<Id>, s: &mut S, e: Entity<Id>) {
        (*self)(c, s, e);
    }
}

pub trait EntityModifier<Id, S>: 'static {
    fn modify(&mut self, &mut EntityManager<Id>, &mut S, Entity<Id>);
}

impl<Id, S> EntityModifier<Id, S> for |&mut EntityManager<Id>, &mut S, Entity<Id>|: 'static {
    fn modify(&mut self, c: &mut EntityManager<Id>, s: &mut S, e: Entity<Id>) {
        (*self)(c, s, e);
    }
}

pub struct Control<'a, Id, S> {
    builders: Vec<Box<EntityBuilder<Id, S> + 'static>>,
    destroyed: Vec<Entity<Id>>,
    modifiers: Vec<(Entity<Id>, Box<EntityModifier<Id, S> + 'static>)>,
}

impl<'a, Id, S> Control<'a, Id, S> {
    pub fn new() -> Control<'a, Id, S> {
        Control {
            builders: Vec::new(),
            destroyed: Vec::new(),
            modifiers: Vec::new(),
        }
    }

    pub fn build(&mut self, builder: Box<EntityBuilder<Id, S> + 'static>) {
        self.builders.push(builder);
    }

    pub fn destroy(&mut self, entity: Entity<Id>) {
        self.destroyed.push(entity);
    }

    pub fn modify(&mut self, entity: Entity<Id>, modifier: Box<EntityModifier<Id, S> + 'static>) {
        self.modifiers.push((entity, modifier));
    }

    pub fn apply(self, entity_manager: &mut EntityManager<Id>, system: &mut S) {
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