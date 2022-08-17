#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    fn new(x: f64, y: f64) -> Coordinate {
        Coordinate { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    id: usize,
    pub position: Coordinate,
    pub neighbors: usize,
}
