#[derive(Clone)]
pub struct Vector {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub position: Vector,
    pub rotation: u16,
    pub dimensions: Vector,
}