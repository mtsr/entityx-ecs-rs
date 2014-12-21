#![feature(phase)]
extern crate anymap;
extern crate test;

#[phase(plugin)] extern crate ecs_macros;

pub use entity::{ EntityManager, Entity, ComponentDatastructure };
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

    use super::{
        Entity,
        Control,
        EntityManager,
        System,
        SystemManager,
        TupAppend, // required for components macro
        ComponentDatastructure,
    };

    use test::Bencher;

    fn test_something() {
        assert!(true);
    }

    #[bench]
    fn bench_something(bencher: &mut Bencher) {
        use test::Bencher;

        let mut system_manager: SystemManager<World1> = SystemManager::new();
        system_manager.register(Sys);

        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        entity_manager.register_component::<Cmp1>(ComponentDatastructure::VecMap);
        entity_manager.register_component::<Cmp2>(ComponentDatastructure::VecMap);
        entity_manager.register_component::<Cmp3>(ComponentDatastructure::VecMap);

        for i in range(0u, 100000u) {
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

    struct Sys;

    impl<Id> System<Id, Sys> for Sys {
        fn update<A>(&mut self, entity_manager: &EntityManager<Id>, control: &mut Control<Id, Sys>, args: &A) where A: Show {

            let mut counter = 0u;

            for (entity, option_cmp2, option_cmp3) in entities_with_components!(entity_manager: without Cmp1 option Cmp2 with Cmp3) {
                counter += 1;
            }
        }
    }
}