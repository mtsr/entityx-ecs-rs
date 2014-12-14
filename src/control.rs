use std::rc::{ Rc };
use std::cell::RefCell;

use entity::{ EntityManager, Entity };

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
    pub fn new() -> Control<'a> {
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

    pub fn apply(self, entity_manager: &Rc<RefCell<EntityManager>>) {
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