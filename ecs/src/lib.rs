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
        system_manager.register(TestSystem::new());

        let mut entity_manager: EntityManager<World1> = EntityManager::new();

        entity_manager.register_component::<Renderable>(ComponentDatastructure::VecMap);
        entity_manager.register_component::<Loud>(ComponentDatastructure::VecMap);
        entity_manager.register_component::<Player>(ComponentDatastructure::HashMap);

        let test_entity1 = entity_manager.create_entity();
        entity_manager.assign_component(&test_entity1, Renderable);
        entity_manager.assign_component(&test_entity1, Loud(1));
        let test_entity2 = entity_manager.create_entity();
        entity_manager.assign_component(&test_entity2, Loud(2));
        entity_manager.assign_component(&test_entity2, Renderable);
        let test_entity3 = entity_manager.create_entity();
        entity_manager.assign_component(&test_entity3, Loud(3));

        entity_manager.destroy_entity(test_entity1);

        let test_entity4 = entity_manager.create_entity();
        entity_manager.assign_component(&test_entity4, Renderable);

        bencher.iter(|| {
            system_manager.update::<UpdateArgs, TestSystem>(&mut entity_manager, &UpdateArgs);
        });
    }

    struct World1;

    #[deriving(Show)]
    struct Renderable;

    #[deriving(Show)]
    struct Loud(int);

    #[deriving(Show)]
    struct Player(uint);

    struct TestSystem {
        num_players: uint,
    }

    impl TestSystem {
        pub fn new() -> TestSystem {
            TestSystem {
                num_players: 0
            }
        }
    }

    #[deriving(Show)]
    struct UpdateArgs;

    impl<Id> System<Id, TestSystem> for TestSystem {
        fn update<A>(&mut self, entity_manager: &EntityManager<Id>, control: &mut Control<Id, TestSystem>, args: &A) where A: Show {

            control.build(box |entity_manager: &mut EntityManager<Id>, system: &mut TestSystem, entity: Entity<Id>| {
                entity_manager.assign_component(&entity, Player(system.num_players));
                system.num_players += 1;
                entity_manager.assign_component(&entity, Renderable);
            });

            for (entity, renderable, option_loud, option_player) in entities_with_components!(entity_manager: with Renderable option Loud option Player) {

                control.modify(entity, box |entity_manager: &mut EntityManager<Id>, system: &mut TestSystem, entity: Entity<Id>| {
                        if let Some(ref mut loud) = entity_manager.get_component_mut::<Loud>(&entity) {
                        loud.0 = 10;
                    };
                });
            }

            for (entity, player) in entities_with_components!(entity_manager: with Player) {
                if let &Player(1) = player {
                } else {
                    continue;
                }
            }

            // for (entity, option_player) in entities_with_components!(entity_manager: with Player) {
            //     control.destroy(entity)
            // }
        }
    }
}