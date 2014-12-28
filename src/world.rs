#![macro_escape]

use std::collections::{ Bitv };

use entity::{ EntityManager, Entity, EntityIterator };
use component::{ ComponentManager, ComponentList, ComponentData };
use system::{ SystemManager, System };

// TODO Add Entity Templates
// TODO Test serialization feasibility

pub struct World<WorldId> {
    entity_manager: EntityManager<WorldId>,
    system_manager: SystemManager<WorldId>,
    component_manager: ComponentManager<WorldId>,
}

impl<WorldId> World<WorldId> {
    pub fn new() -> World<WorldId> {
        let initial_capacity = 256u;

        World {
            entity_manager: EntityManager::new(initial_capacity),
            system_manager: SystemManager::new(),
            component_manager: ComponentManager::new(initial_capacity),
        }
    }

    // *** EntityManager ***

    pub fn create_entity(&mut self) -> Entity<WorldId> {
        let entity = self.entity_manager.create_entity();
        self.component_manager.entity_created(&entity);
        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity<WorldId>) {
        self.component_manager.entity_destroyed(&entity);
        self.entity_manager.destroy_entity(entity)
    }

    pub fn is_valid(&self, entity: &Entity<WorldId>) -> bool {
        self.entity_manager.is_valid(entity)
    }

    pub fn entities(&self) -> EntityIterator<WorldId> {
        self.entity_manager.entities()
    }

    // *** ComponentManager ***

    pub fn register_component<C: 'static>(&mut self, component_list: Box<ComponentList<C> + 'static>) {
        self.component_manager.register_component(component_list)
    }

    pub fn assign_component<C: 'static>(&mut self, entity: &Entity<WorldId>, component: C) {
        assert!(self.is_valid(entity));

        self.component_manager.assign_component(entity, component)
    }

    pub fn has_component<C: 'static>(&self, entity: &Entity<WorldId>) -> bool {
        assert!(self.is_valid(entity));

        self.component_manager.has_component::<C>(entity)
    }

    pub fn get_component<C: 'static>(&self, entity: &Entity<WorldId>) -> Option<&C> {
        assert!(self.is_valid(entity));

        self.component_manager.get_component::<C>(entity)
    }

    pub fn get_component_mut<C: 'static>(&mut self, entity: &Entity<WorldId>) -> Option<&mut C> {
        assert!(self.is_valid(entity));

        self.component_manager.get_component_mut::<C>(entity)
    }

    pub fn get_component_data<C: 'static>(&self) -> &ComponentData<C> {
        self.component_manager.get_component_data::<C>()
    }

    pub fn get_component_data_mut<C: 'static>(&mut self) -> &mut ComponentData<C> {
        self.component_manager.get_component_data_mut::<C>()
    }

    pub fn get_entity_component_mask(&self, entity: &Entity<WorldId>) -> &Bitv {
        self.component_manager.get_entity_component_mask(entity)
    }

    pub fn get_components_length(&self) -> uint {
        self.component_manager.get_components_length()
    }

    // *** SystemManager ***

    pub fn register_system<S>(&mut self, system: S) where S: System<WorldId, S> + 'static {
        self.system_manager.register(system)
    }

    pub fn update_system<A, S>(&mut self, args: &A) where S: System<WorldId, S> + 'static {
        self.system_manager.update::<A,S>(&mut self.entity_manager, &mut self.component_manager, args)
    }
}

// TODO allow with Player(1) style queries.

#[macro_export]
macro_rules! entities_with_components_inner(
    ( $entity_manager:ident, $component_manager:ident, $already:expr : ) => ( $already );
    ( $entity_manager:ident, $component_manager:ident, $already:expr : with $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $entity_manager, $component_manager, $already.and_then(|tuple| {
            let comp = $component_manager.get_component::<$ty>(&tuple.0);
            match comp {
                Some(obj) => Some( tuple.tup_append(obj) ),
                None => None
            }
        } ) : $( $kinds $types )* )
    );
    ( $entity_manager:ident, $component_manager:ident, $already:expr : without $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $entity_manager, $component_manager, $already.and_then(|tuple|
            if let Some(_) = $component_manager.get_component::<$ty>(&tuple.0) {
                None
            } else {
                Some(tuple)
            }
        ) : $( $kinds $types )* )
    );
    ( $entity_manager:ident, $component_manager:ident, $already:expr : option $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $entity_manager, $component_manager, $already.map(|tuple| {
            let comp = $component_manager.get_component::<$ty>(&tuple.0);
            tuple.tup_append( comp )
        } ) : $( $kinds $types )* )
    );
);

#[macro_export]
macro_rules! entities_with_components(
    ( $entity_manager:ident, $component_manager:ident : $( $kinds:ident $types:path )* ) => (
        $entity_manager.entities().filter_map(|entity|
            entities_with_components_inner!($entity_manager, $component_manager, Some((entity,)): $( $kinds $types )* )
        )
    );
);


#[cfg(test)]
mod tests {
    use test::Bencher;
    use std::collections::{ VecMap };
    use super::{ World };

    struct WorldId1;
    struct Cmp1;

    #[test]
    fn create_entity() {
        let mut world:World<WorldId1> = World::new();

        world.create_entity();
    }

    #[test]
    fn destroy_entity() {
        let mut world:World<WorldId1> = World::new();

        let entity = world.create_entity();
        world.destroy_entity(entity);
    }

    #[test]
    fn destroy_entity_with_components() {
        let mut world:World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());

        let entity = world.create_entity();

        world.assign_component(&entity, Cmp1);

        world.destroy_entity(entity);
    }

    #[bench]
    fn bench_create_entity(bencher: &mut Bencher) {
        struct WorldId1;

        let mut world: World<WorldId1> = World::new();
        bencher.iter(|| {
            world.create_entity();
        });
    }

    #[bench]
    fn bench_create_entity_when_1_component(bencher: &mut Bencher) {
        struct WorldId1;

        struct Cmp1;

        let mut world: World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());

        let mut world: World<WorldId1> = World::new();
        bencher.iter(|| {
            world.create_entity();
        });
    }

    #[bench]
    fn bench_create_entity_when_2_components(bencher: &mut Bencher) {
        struct WorldId1;

        struct Cmp1;
        struct Cmp2;

        let mut world: World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());
        world.register_component::<Cmp2>(box VecMap::new());

        let mut world: World<WorldId1> = World::new();
        bencher.iter(|| {
            world.create_entity();
        });
    }

    #[bench]
    fn bench_create_destroy_entity(bencher: &mut Bencher) {
        struct WorldId1;

        let mut world: World<WorldId1> = World::new();

        bencher.iter(|| {
            let entity = world.create_entity();
            world.destroy_entity(entity);
        });
    }    

    #[bench]
    fn bench_create_destroy_entity_when_1_component(bencher: &mut Bencher) {
        struct WorldId1;

        struct Cmp1;

        let mut world: World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());

        bencher.iter(|| {
            let entity = world.create_entity();
            world.destroy_entity(entity);
        });
    }    

    #[bench]
    fn bench_create_destroy_entity_when_2_components(bencher: &mut Bencher) {
        struct WorldId1;

        struct Cmp1;
        struct Cmp2;

        let mut world: World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());
        world.register_component::<Cmp2>(box VecMap::new());

        bencher.iter(|| {
            let entity = world.create_entity();
            world.destroy_entity(entity);
        });
    }
    #[bench]
    fn bench_create_1mm_entities(bencher: &mut Bencher) {
        struct WorldId1;

        let mut world: World<WorldId1> = World::new();
        bencher.iter(|| {
            for _ in range(0u, 1_000_000u) {
                world.create_entity();
            }
        });
    }

    #[bench]
    fn bench_create_destroy_1mm_entities(bencher: &mut Bencher) {
        struct WorldId1;

        let mut world: World<WorldId1> = World::new();

        bencher.iter(|| {
            for _ in range(0u, 1_000_000u) {
                let entity = world.create_entity();
                world.destroy_entity(entity);
            }
        });
    }    

    #[bench]
    fn bench_create_destroy_1mm_entities_when_1_component(bencher: &mut Bencher) {
        struct WorldId1;

        struct Cmp1;

        let mut world: World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());

        bencher.iter(|| {
            for _ in range(0u, 1_000_000u) {
                let entity = world.create_entity();
                world.destroy_entity(entity);
            }
        });
    }    

    #[bench]
    fn bench_create_destroy_1mm_entities_when_2_components(bencher: &mut Bencher) {
        struct WorldId1;

        struct Cmp1;
        struct Cmp2;

        let mut world: World<WorldId1> = World::new();

        world.register_component::<Cmp1>(box VecMap::new());
        world.register_component::<Cmp2>(box VecMap::new());

        bencher.iter(|| {
            for _ in range(0u, 1_000_000u) {
                let entity = world.create_entity();
                world.destroy_entity(entity);
            }
        });
    }
}