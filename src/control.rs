use std::rc::{ Rc };
use std::cell::RefCell;

use entity::{ EntityManager, Entity };

pub trait EntityBuilder<S>: 'static {
    fn build(&mut self, &mut EntityManager, &mut S, Entity);
}

impl<S> EntityBuilder<S> for |&mut EntityManager, &mut S, Entity|: 'static {
    fn build(&mut self, c: &mut EntityManager, s: &mut S, e: Entity) {
        (*self)(c, s, e);
    }
}

pub trait EntityModifier<S>: 'static {
    fn modify(&mut self, &mut EntityManager, &mut S, Entity);
}

impl<S> EntityModifier<S> for |&mut EntityManager, &mut S, Entity|: 'static {
    fn modify(&mut self, c: &mut EntityManager, s: &mut S, e: Entity) {
        (*self)(c, s, e);
    }
}

pub struct Control<'a, S> {
    builders: Vec<Box<EntityBuilder<S> + 'static>>,
    destroyed: Vec<Entity>,
    modifiers: Vec<(Entity, Box<EntityModifier<S> + 'static>)>,
}

impl<'a, S> Control<'a, S> {
    pub fn new() -> Control<'a, S> {
        Control {
            builders: Vec::new(),
            destroyed: Vec::new(),
            modifiers: Vec::new(),
        }
    }

    pub fn build(&mut self, builder: Box<EntityBuilder<S> + 'static>) {
        self.builders.push(builder);
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.destroyed.push(entity);
    }

    pub fn modify(&mut self, entity: Entity, modifier: Box<EntityModifier<S> + 'static>) {
        self.modifiers.push((entity, modifier));
    }

    pub fn apply(self, entity_manager: &Rc<RefCell<EntityManager>>, system: &mut S) {
        let mut entity_manager = entity_manager.borrow_mut();
        for mut builder in self.builders.into_iter() {
            let entity = entity_manager.deref_mut().create_entity();
            builder.build(entity_manager.deref_mut(), system, entity);
        }

        for (entity, mut modifier) in self.modifiers.into_iter() {
            modifier.modify(entity_manager.deref_mut(), system, entity);
        }

        for entity in self.destroyed.into_iter() {
            entity_manager.deref_mut().destroy_entity(entity);
        }
    }
}