mod mesh;
mod material;
mod entity;

pub use mesh::{Mesh, Vertex, create_cube, create_sphere, create_cylinder, create_plane};
pub use material::Material;
pub use entity::{Entity, EntityId};

use std::collections::HashMap;

pub struct Scene {
    entities: HashMap<EntityId, Entity>,
    next_id: u64,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 0,
        }
    }

    #[allow(dead_code)]
    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        self.entities.insert(id, entity);
        id
    }

    pub fn create_entity(&mut self) -> EntityId {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        let entity = Entity::new(id);
        self.entities.insert(id, entity);
        id
    }

    #[allow(dead_code)]
    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    #[allow(dead_code)]
    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities.values_mut()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.next_id = 0;
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
