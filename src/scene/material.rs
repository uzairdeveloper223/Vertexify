use crate::math::Vec3;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(dead_code)]
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

        Some(Self::rgb(r, g, b))
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub roughness: f32,
    pub metallic: f32,
}

#[allow(dead_code)]
impl Material {
    pub fn new(color: Color, roughness: f32, metallic: f32) -> Self {
        Self {
            color,
            roughness: roughness.clamp(0.0, 1.0),
            metallic: metallic.clamp(0.0, 1.0),
        }
    }

    pub fn unlit(color: Color) -> Self {
        Self::new(color, 1.0, 0.0)
    }

    pub fn lambertian(color: Color) -> Self {
        Self::new(color, 0.8, 0.0)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::lambertian(Color::default())
    }
}
