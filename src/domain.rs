#[derive(Clone)]
pub struct Vector {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct FVector {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub position: Vector,
    pub screen_ratio: f32,
    pub rotation: u16,
    pub dimensions: Vector,
}