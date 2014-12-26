use std::collections::{ BinaryHeap };
use std::collections::binary_heap::{ Iter };

// TODO Consider using unsafe for transmuting Option
// use std::mem::transmute;

use std::iter::{ Iterator };
use std::uint;
use std::fmt::{ Show, Formatter, Error };

use std::kinds::marker;

#[deriving(Show,Clone)]
pub struct EntityId {
    index: uint,
    version: uint
}

impl PartialEq for EntityId {
    fn eq(&self, other: &EntityId) -> bool {
        self.index == other.index && self.version == other.version
    }
}

pub struct Entity<WorldId> {
    id: EntityId,
    marker: marker::InvariantType<WorldId>,
}

impl<'a, WorldId> Entity<WorldId> {
    #[inline]
    pub fn index(&self) -> uint {
        self.id.index
    }

    #[inline]
    pub fn version(&self) -> uint {
        self.id.version
    }

    #[inline]
    pub fn id(&self) -> &EntityId {
        &self.id
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

impl<WorldId> PartialEq for Entity<WorldId> {
    fn eq(&self, other: &Entity<WorldId>) -> bool {
        self.id() == other.id()
    }
}

pub struct EntityManager<WorldId> {
    marker: marker::InvariantType<WorldId>,

    next_entity_index: uint,
    free_entity_index_list: BinaryHeap<uint>,

    entity_versions: Vec<uint>,
}

impl<'a, WorldId> EntityManager<WorldId> {
    pub fn new(initial_capacity: uint) -> EntityManager<WorldId> {

        EntityManager {
            marker: marker::InvariantType,

            next_entity_index: 0,
            free_entity_index_list: BinaryHeap::with_capacity(32),

            entity_versions: Vec::from_elem(initial_capacity, 0u),
        }
    }

    fn create_id(&mut self) -> EntityId {
        let index = match self.free_entity_index_list.pop() {
            Some(result) => result,
            None => {
                let result = self.next_entity_index;
                self.next_entity_index += 1;
                result
            }
        };

        if index >= self.entity_versions.capacity() {
            // grow increases capacity in a smart way
            // no reason to specify particular size here
            self.entity_versions.grow(index, 0u);
        }

        let version = self.entity_versions[index];
        EntityId {
            index: index,
            version: version,
        }
    }

    pub fn create_entity(&mut self) -> Entity<WorldId> {
        Entity {
            id: self.create_id(),
            marker: self.marker,
        }
    }

    pub fn destroy_entity(&mut self, entity: Entity<WorldId>) {
        // TODO clear/invalidate component data
        self.entity_versions[entity.index()] += 1;
        self.free_entity_index_list.push(entity.index());
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

            let result = Some(Entity {
                id: EntityId {
                    index: self.index,
                    version: version,
                },
                marker: self.entity_manager.marker,
            });

            self.index += 1;
            return result;
        }

        None::<Entity<WorldId>>
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn create_reuses_index() {
        struct WorldId1;
        let mut entity_manager: EntityManager<WorldId1> = EntityManager::new(256);

        let entity1 = entity_manager.create_entity();
        let entity1_clone = entity1.clone();

        entity_manager.destroy_entity(entity1);

        let entity3 = entity_manager.create_entity();
        assert_eq!(entity3.id.index, entity1_clone.id.index);
        assert_eq!(entity3.id.version, entity1_clone.id.version + 1);
    }
}