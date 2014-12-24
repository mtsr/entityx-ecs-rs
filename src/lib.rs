#![feature(phase)]
#![feature(macro_rules)]
extern crate anymap;
extern crate typemap;
extern crate test;

pub use entity::{ EntityManager, Entity, ComponentList };
pub use system::{ System, SystemManager };
pub use control::{ Control };

pub use tup_append::TupAppend;

mod tup_append;
mod system;
mod entity;
mod control;

#[cfg(test)]
mod tests {
    use std::fmt::Show;
    use std::rand;
    use std::collections::{ HashMap, VecMap };

    use super::{
        // Entity,
        Control,
        EntityManager,
        System,
        SystemManager,
        TupAppend, // required for components macro
    };

    use test::Bencher;

    #[test]
    fn test_something() {
        assert!(true);
    }

    #[bench]
    fn bench_something(bencher: &mut Bencher) {
        let mut system_manager: SystemManager<World1> = SystemManager::new();
        system_manager.register(Sys);

        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        entity_manager.register_component::<Cmp1>(box VecMap::new());
        entity_manager.register_component::<Cmp2>(box VecMap::new());
        entity_manager.register_component::<Cmp3>(box VecMap::new());
        entity_manager.register_component::<Cmp4>(box VecMap::new());
        entity_manager.register_component::<Cmp5>(box HashMap::new());

        for _ in range(0u, 100000u) {
            let entity = entity_manager.create_entity();
            if rand::random::<f32>() > 0.5f32 {
                entity_manager.assign_component(&entity, Cmp1);
            }
            if rand::random::<f32>() > 0.3f32 {
                entity_manager.assign_component(&entity, Cmp2);
            }
            if rand::random::<f32>() > 0.1f32 {
                entity_manager.assign_component(&entity, Cmp3);
            }
            if rand::random::<f32>() > 0.1f32 {
                entity_manager.assign_component(&entity, Cmp4);
            }
            if rand::random::<f32>() > 0.1f32 {
                entity_manager.assign_component(&entity, Cmp5);
            }
        }

        bencher.iter(|| {
            system_manager.update::<uint, Sys>(&mut entity_manager, &0u);
        });
    }

    struct World1;

    #[deriving(Show)]
    struct Cmp1;

    #[deriving(Show)]
    struct Cmp2;

    #[deriving(Show)]
    struct Cmp3;

    #[deriving(Show)]
    struct Cmp4;

    #[deriving(Show)]
    struct Cmp5;

    struct Sys;

    impl<Id> System<Id, Sys> for Sys {
        fn update<A>(&mut self, entity_manager: &EntityManager<Id>, _: &mut Control<Id, Sys>, _: &A) where A: Show {

            let mut counter = 0u;

            for (_, _, _, _, _) in entities_with_components!(entity_manager: without Cmp1 with Cmp2 with Cmp3 with Cmp4 with Cmp5) {
                counter += 1;
            }
        }
    }
}