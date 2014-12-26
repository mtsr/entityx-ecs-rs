#![feature(phase,macro_rules,unboxed_closures)]
extern crate anymap;
extern crate test;

pub use world::World;
pub use entity::{ EntityManager, Entity, ComponentList, ComponentData };
pub use system::{ System, SystemManager };
pub use control::{ Control };

pub use tup_append::TupAppend;

mod world;
mod tup_append;
mod system;
mod entity;
mod control;

#[cfg(test)]
mod tests {
    use std::fmt::Show;
    use std::rand;
    use std::collections::{ Bitv, HashMap, VecMap };

    use super::{
        World,
        EntityManager,
        Entity,
        Control,
        System,
        TupAppend, // required for components macro
    };

    use test::Bencher;

    #[test]
    fn test_something() {
        assert!(true);
    }

    #[bench]
    fn bench_with_macro(bencher: &mut Bencher) {
        let mut world: World<WorldId1> = World::new();

        world.register_system(Sys1);

        world.register_component::<Cmp1>(box VecMap::new());
        world.register_component::<Cmp2>(box VecMap::new());
        world.register_component::<Cmp3>(box VecMap::new());
        world.register_component::<Cmp4>(box VecMap::new());
        world.register_component::<Cmp5>(box HashMap::new());

        for _ in range(0u, 100000u) {
            let entity = world.create_entity();
            if rand::random::<f32>() > 0.5f32 {
                world.assign_component(&entity, Cmp1);
            }
            if rand::random::<f32>() > 0.3f32 {
                world.assign_component(&entity, Cmp2);
            }
            if rand::random::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp3);
            }
            if rand::random::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp4);
            }
            if rand::random::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp5);
            }
        }

        bencher.iter(|| {
            world.update_system::<uint, Sys1>(&0u);
        });
    }

    #[bench]
    fn bench_with_capture(bencher: &mut Bencher) {
        let mut world: World<WorldId1> = World::new();
        world.register_system(Sys2);

        world.register_component::<Cmp1>(box VecMap::new());
        world.register_component::<Cmp2>(box VecMap::new());
        world.register_component::<Cmp3>(box VecMap::new());
        world.register_component::<Cmp4>(box VecMap::new());
        world.register_component::<Cmp5>(box HashMap::new());

        for _ in range(0u, 100000u) {
            let entity = world.create_entity();
            if rand::random::<f32>() > 0.5f32 {
                world.assign_component(&entity, Cmp1);
            }
            if rand::random::<f32>() > 0.3f32 {
                world.assign_component(&entity, Cmp2);
            }
            if rand::random::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp3);
            }
            if rand::random::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp4);
            }
            if rand::random::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp5);
            }
        }

        bencher.iter(|| {
            world.update_system::<uint, Sys2>(&0u);
        });
    }

    struct WorldId1;

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

    struct Sys1;

    impl<WorldId> System<WorldId, Sys1> for Sys1 {
        fn update<A>(&mut self, world: &EntityManager<WorldId>, _: &mut Control<WorldId, Sys1>, _: &A) {

            let mut counter = 0u;

            for (_, _, _, _, _) in entities_with_components!(world: without Cmp1 with Cmp2 with Cmp3 with Cmp4 with Cmp5) {
                counter += 1;
            }
        }
    }

    struct Sys2;

    impl<WorldId> System<WorldId, Sys2> for Sys2 {
        fn update<A>(&mut self, world: &EntityManager<WorldId>, _: &mut Control<WorldId, Sys2>, _: &A) {

            let mut counter = 0u;

            let component_data = (world.get_component_data::<Cmp1>(),)
            .tup_append(world.get_component_data::<Cmp2>())
            .tup_append(world.get_component_data::<Cmp3>())
            .tup_append(world.get_component_data::<Cmp4>())
            .tup_append(world.get_component_data::<Cmp5>());
            let mut with_mask = Bitv::from_elem(world.get_components_length(), false);
            let mut without_mask = Bitv::from_elem(world.get_components_length(), false);
            for tuple in world.entities().filter(|entity| {
                with_mask.set(component_data.1.index, true);
                with_mask.set(component_data.2.index, true);
                with_mask.set(component_data.3.index, true);
                with_mask.set(component_data.4.index, true);

                without_mask.set(component_data.0.index, true);

                let component_mask = world.get_entity_component_mask(entity);

                if with_mask.intersect(component_mask) || without_mask.difference(component_mask) {
                    false
                } else {
                    true
                }
            })
            .filter_map(|entity: Entity<WorldId>| {
                if let Some(component) = component_data.1.list.get(&entity.index()) {
                    return Some((entity, component));
                } else {
                    None
                }
            })
            .filter_map(|tuple| {
                if let Some(component) = component_data.2.list.get(&tuple.0.index()) {
                    return Some(tuple.tup_append(component));
                } else {
                    None
                }
            })
            .filter_map(|tuple| {
                if let Some(component) = component_data.3.list.get(&tuple.0.index()) {
                    return Some(tuple.tup_append(component));
                } else {
                    None
                }
            })
            .filter_map(|tuple| {
                if let Some(component) = component_data.4.list.get(&tuple.0.index()) {
                    return Some(tuple.tup_append(component));
                } else {
                    None
                }
            }) {
                // println!("{}", tuple);
                counter += 1;
            }
        }
    }
}