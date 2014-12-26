use entity::{ EntityManager, Entity };

pub trait EntityBuilder<WorldId, S>: 'static {
    fn build(&mut self, &mut EntityManager<WorldId>, &mut S, Entity<WorldId>);
}

impl<WorldId, S> EntityBuilder<WorldId, S> for |&mut EntityManager<WorldId>, &mut S, Entity<WorldId>|: 'static {
    fn build(&mut self, c: &mut EntityManager<WorldId>, s: &mut S, e: Entity<WorldId>) {
        (*self)(c, s, e);
    }
}

pub trait EntityModifier<WorldId, S>: 'static {
    fn modify(&mut self, &mut EntityManager<WorldId>, &mut S, Entity<WorldId>);
}

impl<WorldId, S> EntityModifier<WorldId, S> for |&mut EntityManager<WorldId>, &mut S, Entity<WorldId>|: 'static {
    fn modify(&mut self, c: &mut EntityManager<WorldId>, s: &mut S, e: Entity<WorldId>) {
        (*self)(c, s, e);
    }
}

pub struct Control<'a, WorldId, S> {
    builders: Vec<Box<EntityBuilder<WorldId, S> + 'static>>,
    destroyed: Vec<Entity<WorldId>>,
    modifiers: Vec<(Entity<WorldId>, Box<EntityModifier<WorldId, S> + 'static>)>,
}

impl<'a, WorldId, S> Control<'a, WorldId, S> {
    pub fn new() -> Control<'a, WorldId, S> {
        Control {
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