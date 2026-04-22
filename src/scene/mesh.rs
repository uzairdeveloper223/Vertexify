use crate::math::{Vec3, Vec2, BoundingBox};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self { position, normal, uv }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    bounds: Option<BoundingBox>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            bounds: None,
        }
    }

    #[allow(dead_code)]
    pub fn bounds(&mut self) -> BoundingBox {
        if let Some(bounds) = self.bounds {
            return bounds;
        }

        let positions: Vec<Vec3> = self.vertices.iter().map(|v| v.position).collect();
        let bounds = BoundingBox::from_points(&positions);
        self.bounds = Some(bounds);
        bounds
    }

    #[allow(dead_code)]
    pub fn calculate_normals(&mut self) {
        for vertex in &mut self.vertices {
            vertex.normal = Vec3::ZERO;
        }

        for i in (0..self.indices.len()).step_by(3) {
            let i0 = self.indices[i] as usize;
            let i1 = self.indices[i + 1] as usize;
            let i2 = self.indices[i + 2] as usize;

            let v0 = self.vertices[i0].position;
            let v1 = self.vertices[i1].position;
            let v2 = self.vertices[i2].position;

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(edge2);

            self.vertices[i0].normal += normal;
            self.vertices[i1].normal += normal;
            self.vertices[i2].normal += normal;
        }

        for vertex in &mut self.vertices {
            vertex.normal = vertex.normal.normalize();
        }
    }

    pub fn transform(&mut self, matrix: &glam::Mat4) {
        let normal_matrix = matrix.inverse().transpose();

        for vertex in &mut self.vertices {
            let pos = matrix.transform_point3(vertex.position);
            vertex.position = pos;

            let normal = normal_matrix.transform_vector3(vertex.normal);
            vertex.normal = normal.normalize();
        }

        self.bounds = None;
    }
}

pub fn create_cube(width: f32, height: f32, depth: f32) -> Mesh {
    let w = width * 0.5;
    let h = height * 0.5;
    let d = depth * 0.5;

    let vertices = vec![
        Vertex::new(Vec3::new(-w, -h, -d), Vec3::NEG_Z, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(w, -h, -d), Vec3::NEG_Z, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(w, h, -d), Vec3::NEG_Z, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-w, h, -d), Vec3::NEG_Z, Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-w, -h, d), Vec3::Z, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(w, -h, d), Vec3::Z, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(w, h, d), Vec3::Z, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-w, h, d), Vec3::Z, Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-w, -h, -d), Vec3::NEG_X, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(-w, h, -d), Vec3::NEG_X, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-w, h, d), Vec3::NEG_X, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-w, -h, d), Vec3::NEG_X, Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(w, -h, -d), Vec3::X, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(w, h, -d), Vec3::X, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(w, h, d), Vec3::X, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(w, -h, d), Vec3::X, Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-w, -h, -d), Vec3::NEG_Y, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(w, -h, -d), Vec3::NEG_Y, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(w, -h, d), Vec3::NEG_Y, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-w, -h, d), Vec3::NEG_Y, Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-w, h, -d), Vec3::Y, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(w, h, -d), Vec3::Y, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(w, h, d), Vec3::Y, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-w, h, d), Vec3::Y, Vec2::new(0.0, 1.0)),
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0,
        4, 6, 5, 6, 4, 7,
        8, 9, 10, 10, 11, 8,
        12, 14, 13, 14, 12, 15,
        16, 17, 18, 18, 19, 16,
        20, 22, 21, 22, 20, 23,
    ];

    Mesh::new(vertices, indices)
}

pub fn create_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for segment in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let x = sin_phi * cos_theta;
            let y = cos_phi;
            let z = sin_phi * sin_theta;

            let position = Vec3::new(x, y, z) * radius;
            let normal = Vec3::new(x, y, z);
            let uv = Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32);

            vertices.push(Vertex::new(position, normal, uv));
        }
    }

    for ring in 0..rings {
        for segment in 0..segments {
            let current = ring * (segments + 1) + segment;
            let next = current + segments + 1;

            indices.push(current);
            indices.push(next);
            indices.push(current + 1);

            indices.push(current + 1);
            indices.push(next);
            indices.push(next + 1);
        }
    }

    Mesh::new(vertices, indices)
}

pub fn create_cylinder(radius: f32, height: f32, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let half_height = height * 0.5;

    for i in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let x = radius * cos_theta;
        let z = radius * sin_theta;
        let normal = Vec3::new(cos_theta, 0.0, sin_theta);
        let u = i as f32 / segments as f32;

        vertices.push(Vertex::new(Vec3::new(x, -half_height, z), normal, Vec2::new(u, 0.0)));
        vertices.push(Vertex::new(Vec3::new(x, half_height, z), normal, Vec2::new(u, 1.0)));
    }

    for i in 0..segments {
        let base = i * 2;
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);

        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    vertices.push(Vertex::new(Vec3::new(0.0, -half_height, 0.0), Vec3::NEG_Y, Vec2::new(0.5, 0.5)));
    vertices.push(Vertex::new(Vec3::new(0.0, half_height, 0.0), Vec3::Y, Vec2::new(0.5, 0.5)));

    let bottom_center = vertices.len() as u32 - 2;
    let top_center = vertices.len() as u32 - 1;

    for i in 0..segments {
        let base = i * 2;
        indices.push(bottom_center);
        indices.push(base);
        indices.push(base + 2);

        indices.push(top_center);
        indices.push(base + 3);
        indices.push(base + 1);
    }

    Mesh::new(vertices, indices)
}

pub fn create_plane(width: f32, depth: f32) -> Mesh {
    let w = width * 0.5;
    let d = depth * 0.5;

    let vertices = vec![
        Vertex::new(Vec3::new(-w, 0.0, -d), Vec3::Y, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(w, 0.0, -d), Vec3::Y, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(w, 0.0, d), Vec3::Y, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-w, 0.0, d), Vec3::Y, Vec2::new(0.0, 1.0)),
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    Mesh::new(vertices, indices)
}
