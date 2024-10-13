use glam::Vec3;

pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3) -> Camera {
        Camera {
            eye,
            center,
            up: Vec3 {
                x: 0.,
                y: 1.,
                z: 0.,
            },
        }
    }
}
