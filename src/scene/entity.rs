use crate::math::Transform;
use crate::scene::{Mesh, Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct Entity {
    pub id: EntityId,
    pub transform: Transform,
    pub mesh: Option<Mesh>,
    pub material: Material,
    pub visible: bool,
}

#[allow(dead_code)]
impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            transform: Transform::default(),
            mesh: None,
            material: Material::default(),
            visible: true,
        }
    }

    pub fn with_mesh(mut self, mesh: Mesh) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}
