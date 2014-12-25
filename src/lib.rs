#![feature(phase)]
#![feature(macro_rules)]
extern crate anymap;
extern crate test;

pub use entity::{ EntityManager, Entity, ComponentList, ComponentData };
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
    use std::collections::{ Bitv, HashMap, VecMap };

    use super::{
        Entity,
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
    fn bench_with_macro(bencher: &mut Bencher) {
        let mut system_manager: SystemManager<World1> = SystemManager::new();
        system_manager.register(Sys1);

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
            system_manager.update::<uint, Sys1>(&mut entity_manager, &0u);
        });
    }

    #[bench]
    fn bench_with_capture(bencher: &mut Bencher) {
        let mut system_manager: SystemManager<World1> = SystemManager::new();
        system_manager.register(Sys2);

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
            system_manager.update::<uint, Sys2>(&mut entity_manager, &0u);
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

    struct Sys1;

    impl<Id> System<Id, Sys1> for Sys1 {
        fn update<A>(&mut self, entity_manager: &EntityManager<Id>, _: &mut Control<Id, Sys1>, _: &A) where A: Show {

            let mut counter = 0u;

            for (_, _, _, _, _) in entities_with_components!(entity_manager: without Cmp1 with Cmp2 with Cmp3 with Cmp4 with Cmp5) {
                counter += 1;
            }
        }
    }

    struct Sys2;

    impl<Id> System<Id, Sys2> for Sys2 {
        fn update<A>(&mut self, entity_manager: &EntityManager<Id>, _: &mut Control<Id, Sys2>, _: &A) where A: Show {

            let mut counter = 0u;

            let component_data = (entity_manager.get_component_data::<Cmp1>(),)
            .tup_append(entity_manager.get_component_data::<Cmp2>())
            .tup_append(entity_manager.get_component_data::<Cmp3>())
            .tup_append(entity_manager.get_component_data::<Cmp4>())
            .tup_append(entity_manager.get_component_data::<Cmp5>());
            let mut with_mask = Bitv::from_elem(entity_manager.get_components_length(), false);
            let mut without_mask = Bitv::from_elem(entity_manager.get_components_length(), false);
            for tuple in entity_manager.entities().filter(|entity| {
                with_mask.set(component_data.1.index, true);
                with_mask.set(component_data.2.index, true);
                with_mask.set(component_data.3.index, true);
                with_mask.set(component_data.4.index, true);

                without_mask.set(component_data.0.index, true);

                let component_mask = entity_manager.get_entity_component_mask(entity);

                if with_mask.intersect(component_mask) || without_mask.difference(component_mask) {
                    false
                } else {
                    true
                }
            })
            .filter_map(|entity: Entity<Id>| {
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