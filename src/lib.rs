#![feature(test,collections,core)]

extern crate anymap;
extern crate test;

pub use world::World;
pub use entity::{ EntityManager, Entity };
pub use system::{ System, SystemManager };
pub use control::{ Control };
pub use component::{ ComponentManager, ComponentList, ComponentData };

pub use tup_append::TupAppend;

mod world;
mod tup_append;
mod system;
mod entity;
mod control;
mod component;

#[cfg(test)]
mod tests {
    use std::rand::{ Rng, XorShiftRng };
    use std::collections::{ BitVec, HashMap, VecMap };

    use super::{
        World,
        EntityManager,
        ComponentManager,
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
    fn bench_iterate_over_100k_entities_with_5_components(bencher: &mut Bencher) {
        let mut rng = XorShiftRng::new_unseeded();

        let mut world: World<WorldId1> = World::new();

        world.register_system(Sys);

        world.register_component::<Cmp1>(Box::new(VecMap::new()));
        world.register_component::<Cmp2>(Box::new(VecMap::new()));
        world.register_component::<Cmp3>(Box::new(VecMap::new()));
        world.register_component::<Cmp4>(Box::new(VecMap::new()));
        world.register_component::<Cmp5>(Box::new(HashMap::new()));

        for _ in range(0usize, 100000usize) {
            let entity = world.create_entity();
            if rng.gen::<f32>() > 0.5f32 {
                world.assign_component(&entity, Cmp1);
            }
            if rng.gen::<f32>() > 0.3f32 {
                world.assign_component(&entity, Cmp2);
            }
            if rng.gen::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp3);
            }
            if rng.gen::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp4);
            }
            if rng.gen::<f32>() > 0.1f32 {
                world.assign_component(&entity, Cmp5);
            }
        }

        bencher.iter(|| {
            world.update_system::<usize, Sys>(&0usize);
        });
    }

    struct WorldId1;

    #[derive(Debug)]
    struct Cmp1;

    #[derive(Debug)]
    struct Cmp2;

    #[derive(Debug)]
    struct Cmp3;

    #[derive(Debug)]
    struct Cmp4;

    #[derive(Debug)]
    struct Cmp5;

    struct Sys;

    impl<WorldId> System<WorldId, Sys> for Sys {
        fn update<A>(&mut self, entity_manager: &EntityManager<WorldId>, component_manager: &ComponentManager<WorldId>, _: &mut Control<WorldId, Sys>, _: &A) {

            let mut counter = 0usize;

            let component_data = (component_manager.get_component_data::<Cmp1>(),)
            .tup_append(component_manager.get_component_data::<Cmp2>())
            .tup_append(component_manager.get_component_data::<Cmp3>())
            .tup_append(component_manager.get_component_data::<Cmp4>())
            .tup_append(component_manager.get_component_data::<Cmp5>());

            let mut with_mask = BitVec::from_elem(component_manager.get_components_length(), false);
            let mut without_mask = BitVec::from_elem(component_manager.get_components_length(), false);

            for tuple in entity_manager.entities().filter_map(|entity| {
                with_mask.set(component_data.1.index, true);
                with_mask.set(component_data.2.index, true);
                with_mask.set(component_data.3.index, true);
                with_mask.set(component_data.4.index, true);

                without_mask.set(component_data.0.index, true);

                let component_mask = component_manager.get_entity_component_mask(&entity);

                if with_mask.intersect(component_mask) || without_mask.difference(component_mask) {
                    None
                } else {
                    let index = &entity.index();
                    Some((entity,
                        component_data.1.list.get(index).unwrap(),
                        component_data.2.list.get(index).unwrap(),
                        component_data.3.list.get(index).unwrap(),
                        component_data.4.list.get(index).unwrap(),
                    ))
                }
            }) {
                // println!("{}", tuple);
                counter += 1;
            }
        }
    }
}