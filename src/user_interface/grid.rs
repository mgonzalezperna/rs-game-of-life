use crate::gol::models::{Cell, Coordinate};
use tui::{
    style::Color,
    widgets::canvas::{Painter, Shape},
};

/// Shape to draw a world map with the given resolution and color
#[derive(Debug, Clone)]
pub struct Grid {
    pub cells: Vec<Cell>,
    pub color: Color,
}

impl Default for Grid {
    fn default() -> Grid {
        Grid {
            cells: vec![],
            color: Color::Reset,
        }
    }
}

///impl Grid {
///    fn add_cell(&self, Cell) {}
///}

impl Shape for Grid {
    fn draw(&self, painter: &mut Painter) {
        for cell in &self.cells {
            let position = &cell.position;
            if let Some((x, y)) = painter.get_point(position.x, position.y) {
                painter.paint(x, y, self.color);
            }
        }
    }
}
