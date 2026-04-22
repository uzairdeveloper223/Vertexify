use crate::math::{Vec3, Mat4};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let radius = (self.position - self.target).length();
        
        let offset = self.position - self.target;
        let yaw = offset.z.atan2(offset.x);
        let pitch = (offset.y / radius).asin();

        let new_yaw = yaw + delta_x;
        let new_pitch = (pitch + delta_y).clamp(-1.5, 1.5);

        let x = radius * new_pitch.cos() * new_yaw.cos();
        let y = radius * new_pitch.sin();
        let z = radius * new_pitch.cos() * new_yaw.sin();

        self.position = self.target + Vec3::new(x, y, z);
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward);

        let offset = right * delta_x + up * delta_y;
        self.position += offset;
        self.target += offset;
    }

    pub fn zoom(&mut self, delta: f32) {
        let direction = (self.target - self.position).normalize();
        let distance = (self.position - self.target).length();
        let new_distance = (distance - delta).max(1.0);
        
        self.position = self.target - direction * new_distance;
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
