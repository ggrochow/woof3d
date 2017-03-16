use std::fmt;
use std::fmt::Write;
use std::collections::HashMap;
use std::collections::BTreeSet;
use rand::Rng;
use rand;
use world::Wall;

pub struct Maze {
    pub height: usize,
    pub width: usize,
    pub cells: Vec<Cell>,
    pub links: HashMap<(usize, usize), BTreeSet<(usize, usize)>>,
}

impl Maze {
    pub fn generate_maze(height: usize, width: usize) -> Self {
        let mut maze = Maze::blank(height, width);

        let mut cell = maze.get_random_cell();
        let mut unvisited = maze.size() - 1;

        while unvisited > 0 {
            let neighbours = maze.get_cell_neighbours(&cell);
            let neighbour = rand::thread_rng().choose(&neighbours).unwrap();

            if !maze.links.contains_key(&(neighbour.x, neighbour.y)) {
                maze.link_cells(&cell, &neighbour);
                unvisited -= 1;
            }

            cell = neighbour.clone();
        }

        maze
    }

    pub fn blank(height: usize, width: usize) -> Self {
        let mut cell_raster = Vec::with_capacity(height * width);

        for y in 0..height {
            for x in 0..width {
                cell_raster.push(Cell::new(x, y));
            }
        }

        Maze {
            height: height,
            width: width,
            cells: cell_raster,
            links: HashMap::new(),
        }
    }

    pub fn link_coords(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        match self.links.contains_key(&(x1, y1)) {
            true => {
                self.links.get_mut(&(x1, y1)).unwrap().insert((x2, y2));
            },
            false => {
                let mut set: BTreeSet<(usize, usize)> = BTreeSet::new();
                set.insert((x2, y2));
                self.links.insert((x1, y1), set);
            }
        }
    }

    pub fn link_cells(&mut self, cell1: &Cell, cell2: &Cell) {
        self.link_pair(cell1.x, cell1.y, cell2.x, cell2.y);
    }

    pub fn link_pair(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        self.link_coords(x1, y1, x2, y2);
        self.link_coords(x2, y2, x1, y1);
    }


    fn get_random_cell(&self) -> Cell {
        rand::thread_rng().choose(&self.cells).unwrap().clone()
    }

    fn size(&self) -> usize {
        self.height * self.width
    }

    pub fn get_cell_in_direction(&self, cell: &Cell, direction: &Direction) -> Option<&Cell> {
        if let Some((x, y)) = self.get_inbounds_coords_in_direction(cell.x, cell.y, direction) {
            self.get_cell(x, y)
        } else {
            None
        }
    }

    pub fn get_inbounds_coords_in_direction(&self, x: usize, y: usize, direction: &Direction) -> Option<(usize, usize)> {
        let mut x_cursor = Some(0);
        let mut y_cursor = Some(0);

        match *direction {
            Direction::N => y_cursor = y.checked_sub(1),
            Direction::S => y_cursor = y.checked_add(1),
            Direction::E => x_cursor = x.checked_add(1),
            Direction::W => x_cursor = x.checked_sub(1),
        };

        if x_cursor.is_none() || y_cursor.is_none() {
            None
        } else {
            let mut x_res = x_cursor.unwrap();
            let mut y_res = y_cursor.unwrap();

            match direction {
                &Direction::N | &Direction::S => {
                    x_res = x;
                },
                &Direction::E | &Direction::W => {
                    y_res = y;
                },
            }

            if x_res >= self.width || y_res >= self.height {
                None
            } else {
                Some((x_res, y_res))
            }
        }
    }

    pub fn get_cell_neighbours(&self, cell: &Cell) -> Vec<Cell> {
        let mut out = Vec::with_capacity(4);

        for dir in Direction::all().into_iter() {
            if let Some(cell) = self.get_cell_in_direction(cell, &dir) {
                out.push(cell.clone());
            }
        }
        out
    }

    fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(y * self.width + x)
    }

    pub fn to_walls(&self, scale: usize) -> Vec<Wall> {
        let mut out = Vec::new();

        for cell in &self.cells {
            let x = cell.x * scale;
            let y = cell.y * scale;
            let offset = 1 * scale;

            if !self.is_linked(&cell, &Direction::N) {
                out.push(Wall::new_usize(x, y, x + offset, y));
            }

            if !self.is_linked(&cell, &Direction::S) {
                out.push(Wall::new_usize(x, y + offset, x + offset, y + offset));
            }

            if !self.is_linked(&cell, &Direction::E) {
                out.push(Wall::new_usize(x + offset, y, x + offset, y + offset));
            }

            if !self.is_linked(&cell, &Direction::W) {
                out.push(Wall::new_usize(x, y, x, y + offset));
            }
        }

        out
    }

    pub fn is_linked(&self, cell: &Cell, direction: &Direction) -> bool {
        if let Some((x, y)) = self.get_inbounds_coords_in_direction(cell.x, cell.y, direction) {
            self.is_linked_coords(cell.x, cell.y, x, y)
        } else {
            false 
        }
    }

    pub fn is_linked_coords(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        if let Some(set) = self.links.get(&(x1, y1)) {
            if let Some(_) = set.get(&(x2, y2)) {
                return true;
            }
        }

        false
    }

}


#[derive(Debug, Clone)]
pub struct Cell {
    pub x: usize,
    pub y: usize
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Cell {
            x: x,
            y: y,
        }
    }
}

#[derive(Debug)]
pub enum Direction {
    N, W, S, E
}

impl Direction {

    pub fn all() -> [Direction; 4] {
        [Direction::N, Direction::W, Direction::S, Direction::E]
    }

}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "+".to_string();
        for _ in 0..self.width {
            write!(output, "---+").unwrap();
        }
        write!(output, "\n").unwrap();

        for y in 0..self.height {
            let mut top = "|".to_string();
            let mut bottom = "+".to_string();

            for x in 0..self.width {
                let cell = self.get_cell(x, y).unwrap();
                write!(top, "   ").unwrap();
                if self.is_linked(&cell, &Direction::E) {
                    write!(top, " ").unwrap();
                } else {
                    write!(top, "|").unwrap();
                }

                if self.is_linked(&cell, &Direction::S) {
                    write!(bottom, "   ").unwrap();
                } else {
                    write!(bottom, "---").unwrap();
                }

                write!(bottom, "+").unwrap();
            }

            write!(output, "{}\n", top).unwrap();
            write!(output, "{}\n", bottom).unwrap();
        }

        write!(f, "{}", output)
    }
}
