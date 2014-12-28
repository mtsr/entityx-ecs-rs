use std::collections::{ RingBuf };
use std::collections::ring_buf::{ Iter };

// TODO Consider using unsafe for transmuting Option
// use std::mem::transmute;

use std::iter::{ Iterator };
use std::uint;
use std::fmt::{ Show, Formatter, Error };

use std::kinds::marker;

// use 1/4 of uint bits for version rest for index
const INDEX_BITS: uint = uint::BITS / 4 * 3;
const INDEX_MASK: uint = (1 << INDEX_BITS) - 1;

// Necessary to ensure enough versions in limited number of bits
// eg 8 bits = only 256 versions
// By reusing in FIFO order and ensuring MINIMUM_FREE_ENTITY_INDICES
// even destroying and creating a single entity will still allow
// 256 * MINIMUM_FREE_ENTITY_INDICES entities to be created before
// version wraps around
const MINIMUM_FREE_ENTITY_INDICES: uint = 1000;

pub struct Entity<WorldId> {
    id: uint,
    marker: marker::InvariantType<WorldId>,
}

impl<'a, WorldId> Entity<WorldId> {
    pub fn new(marker: marker::InvariantType<WorldId>, index: uint, version: uint) -> Entity<WorldId> {
        debug_assert!(index & INDEX_MASK as uint == index);
        debug_assert!(version & !INDEX_MASK as uint == version);

        Entity {
            id: index | (version << INDEX_BITS),
            marker: marker,
        }
    }

    #[inline]
    pub fn index(&self) -> uint {
        self.id & INDEX_MASK
    }

    #[inline]
    pub fn version(&self) -> uint {
        self.id >> INDEX_BITS
    }
}

impl<WorldId> PartialEq for Entity<WorldId> {
    fn eq(&self, other: &Entity<WorldId>) -> bool {
        self.id == other.id
    }
}

impl<WorldId> Clone for Entity<WorldId> {
    fn clone(&self) -> Self {
        Entity {
            id: self.id.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<WorldId> Show for Entity<WorldId> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        self.id.fmt(formatter)
    }
}

pub struct EntityManager<WorldId> {
    marker: marker::InvariantType<WorldId>,

    next_entity_index: uint,

    // FIFO
    free_entity_index_list: RingBuf<uint>,

    entity_versions: Vec<uint>,
}

impl<'a, WorldId> EntityManager<WorldId> {
    pub fn new(initial_capacity: uint) -> EntityManager<WorldId> {

        EntityManager {
            marker: marker::InvariantType,

            next_entity_index: 0,
            free_entity_index_list: RingBuf::with_capacity(MINIMUM_FREE_ENTITY_INDICES),

            entity_versions: Vec::from_elem(initial_capacity, 0u),
        }
    }

    pub fn create_entity(&mut self) -> Entity<WorldId> {
        let index = if self.free_entity_index_list.len() > MINIMUM_FREE_ENTITY_INDICES {
            // FIFO
            self.free_entity_index_list.pop_front().unwrap()
        } else {
            let index = self.next_entity_index;
            self.next_entity_index += 1;
            index
        };

        if index >= self.entity_versions.len() {
            // grow increases capacity in a smart way
            // no reason to specify particular size here
            self.entity_versions.grow(index, 0u);
        }

        let version = self.entity_versions[index];
        Entity::new(self.marker.clone(), index, version)
    }
    pub fn destroy_entity(&mut self, entity: Entity<WorldId>) {
        // TODO clear/invalidate component data
        self.entity_versions[entity.index()] += 1;
        // FIFO
        self.free_entity_index_list.push_back(entity.index());
    }

    pub fn is_valid(&self, entity: &Entity<WorldId>) -> bool {
        entity.index() < self.next_entity_index
        && entity.version() == self.entity_versions[entity.index()]
    }

    pub fn entities(&self) -> EntityIterator<WorldId> {
        EntityIterator {
            entity_manager: self,
            last_entity_index: self.next_entity_index - 1, // last valid entity index
            index: 0,
            free_entity_index_list: self.free_entity_index_list.iter(),
        }
    }
}

pub struct EntityIterator<'a, WorldId: 'a> {
    entity_manager: &'a EntityManager<WorldId>,
    last_entity_index: uint,
    index: uint,
    free_entity_index_list: Iter<'a, uint>,
}

impl<'a, WorldId> Iterator<Entity<WorldId>> for EntityIterator<'a, WorldId> {
    fn next(&mut self) -> Option<Entity<WorldId>> {
        // for all valid entity indexes
        while self.index <= self.last_entity_index {
            let mut free_entity_index = -1;

            // find if the index is in the free_entity_index_list
            while free_entity_index < self.index {
                free_entity_index = match self.free_entity_index_list.next() {
                    Some(x) => *x,
                    None => uint::MAX,
                }
            }

            if free_entity_index == self.index {
                self.index += 1;
                continue;
            }

            let version = self.entity_manager.entity_versions[self.index];

            let result = Some(Entity::new(self.entity_manager.marker.clone(), self.index, version));

            self.index += 1;
            return result;
        }

        None::<Entity<WorldId>>
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::{ EntityManager };

    #[test]
    fn created_entity_is_valid() {
        struct WorldId1;
        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        let entity = entity_manager.create_entity();
        assert!(entity_manager.is_valid(&entity));
    }

    #[test]
    fn deleted_entity_is_invalid() {
        struct WorldId1;
        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        let entity1 = entity_manager.create_entity();
        let entity1_clone = entity1.clone();

        assert!(entity_manager.is_valid(&entity1_clone));
        entity_manager.destroy_entity(entity1);
        assert!(!entity_manager.is_valid(&entity1_clone));
    }

    #[bench]
    fn create_1mm_entities(bencher: &mut Bencher) {

        struct WorldId1;

        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);
       bencher.iter(|| {
            for _ in range(0u, 1_000_000u) {
                entity_manager.create_entity();
            }
        });
    }

    #[bench]
    fn create_destroy_1mm_entities(bencher: &mut Bencher) {
        struct WorldId1;

        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        bencher.iter(|| {
            for _ in range(0u, 1_000_000u) {
                let entity = entity_manager.create_entity();
                entity_manager.destroy_entity(entity);
            }
        });
    }
}