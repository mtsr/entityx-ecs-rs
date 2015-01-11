use std::collections::{ RingBuf };
use std::collections::ring_buf::{ Iter };

use std::iter::{ Iterator, repeat };
use std::{ usize };
use std::fmt::{ Show, Formatter, Error };

use std::marker::{ InvariantType };

// TODO get rid of usize here
// use 1/4 of usize bits for version rest for index
const INDEX_BITS: usize = usize::BITS / 4 * 3;
const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;

// Necessary to ensure enough versions in limited number of bits
// eg 8 bits = only 256 versions
// By reusing in FIFO order and ensuring MINIMUM_FREE_ENTITY_INDICES
// even destroying and creating a single entity will still allow
// 256 * MINIMUM_FREE_ENTITY_INDICES entities to be created before
// version wraps around
const MINIMUM_FREE_ENTITY_INDICES: usize = 1000;

pub struct Entity<WorldId> {
    id: usize,
    marker: InvariantType<WorldId>,
}

impl<'a, WorldId> Entity<WorldId> {
    pub fn new(marker: InvariantType<WorldId>, index: usize, version: usize) -> Entity<WorldId> {
        debug_assert!(index & INDEX_MASK as usize == index);
        debug_assert!(version & !INDEX_MASK as usize == version);

        Entity {
            id: index | (version << INDEX_BITS),
            marker: marker,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id & INDEX_MASK
    }

    #[inline]
    pub fn version(&self) -> usize {
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
    marker: InvariantType<WorldId>,

    next_entity_index: usize,

    // FIFO
    free_entity_index_list: RingBuf<usize>,

    entity_versions: Vec<usize>,
}

impl<'a, WorldId> EntityManager<WorldId> {
    pub fn new(initial_capacity: usize) -> EntityManager<WorldId> {

        EntityManager {
            marker: InvariantType,

            next_entity_index: 0,
            free_entity_index_list: RingBuf::with_capacity(MINIMUM_FREE_ENTITY_INDICES),

            entity_versions: repeat(0us).take(initial_capacity).collect(),
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

        let entity_versions_len = self.entity_versions.len();
        if index >= entity_versions_len {
            // grow increases capacity in a smart way
            // no reason to specify particular size here
            self.entity_versions.extend(repeat(0us).take(entity_versions_len));
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
    last_entity_index: usize,
    index: usize,
    free_entity_index_list: Iter<'a, usize>,
}

impl<'a, WorldId> Iterator for EntityIterator<'a, WorldId> {
    type Item = Entity<WorldId>;

    fn next(&mut self) -> Option<Entity<WorldId>> {
        // for all valid entity indexes
        while self.index <= self.last_entity_index {
            let mut free_entity_index = -1;

            // find if the index is in the free_entity_index_list
            while free_entity_index < self.index {
                free_entity_index = match self.free_entity_index_list.next() {
                    Some(x) => *x,
                    None => usize::MAX,
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
            for _ in range(0us, 1_000_000us) {
                entity_manager.create_entity();
            }
        });
    }

    #[bench]
    fn create_destroy_1mm_entities(bencher: &mut Bencher) {
        struct WorldId1;

        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        bencher.iter(|| {
            for _ in range(0us, 1_000_000us) {
                let entity = entity_manager.create_entity();
                entity_manager.destroy_entity(entity);
            }
        });
    }
}