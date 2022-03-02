#[derive(Default, Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Quat4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}
