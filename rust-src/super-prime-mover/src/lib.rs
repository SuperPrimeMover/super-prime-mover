// priorities: bas droite gauche haut

mod array2d;
use array2d::Array2D;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    South,
    West,
    East,
    North,
}

impl Orientation {
    pub fn to_vector(&self) -> (isize, isize) {
        match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::West => (-1, 0),
            Self::East => (1, 0),
        }
    }

    pub fn step(&self, a: usize, b: usize) -> (usize, usize) {
        let vector = self.to_vector();
        (
            (a as isize + vector.0) as usize,
            (b as isize + vector.1) as usize,
        )
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BoardIcon {
    Green,
    Red,
    Blue,
    Other { png_data: Vec<u8> },
}

// Signal { orientation: South }
// Wire { wire1: South, wire2: North }
//

// TODO: Find a way to make Tile more lightweight.
// BODY: It'd be nice if Tile was Copy, as that'd make manipulation a lot
// BODY: easier. The problem is, the Board subtype has a lot of heavy
// BODY: information in it.
// BODY:
// BODY: A potential solution is to have Board contain a simple index to the
// BODY: subboard, and keep the subboards in a separate vector. The subboards
// BODY: would never get evicted (as they're necessary to handle undo anyways).
#[derive(Debug, Clone)]
pub enum Tile {
    Empty,
    Unusable {
        /// If true, this tile is a vanilla game "broken" tile on which nothing
        /// can get placed. If false, it's an "invisible" tile that may be used
        /// as a pseudo input.
        broken: bool,
    },
    Wire {
        // wire1, wire2
        slow: bool,
    },
    Bridge,
    Joiner {
        orientation: Orientation,
    },
    Cloner,
    Sorter {
        orientation: Orientation,
        reversed: bool,
    },
    Deleter,
    Flipflop {
        orientation: Orientation,
        reversed: bool,
    },
    Incrementer {
        reversed: bool,
    },
    Button {
        orientation: Orientation,
    },
    Lock {
        locked: bool,
    },
    SubBoard {
        contents: Board,
        icon: BoardIcon,
    },
}

impl Default for Tile {
    fn default() -> Self {
        Self::Empty
    }
}

impl Tile {
    pub fn max_connections(&self) -> usize {
        match self {
            Tile::Button { .. } => 2,
            Tile::Wire { .. } => 2,
            Tile::Lock { .. } => 2,
            Tile::Cloner => 3,
            Tile::Flipflop { .. } => 3,
            Tile::Sorter { .. } => 3,
            Tile::SubBoard { .. } => 4,
            Tile::Joiner { .. } => 4,
            Tile::Incrementer { .. } => 4,
            Tile::Deleter => 4,
            Tile::Bridge => 4,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    tiles: Array2D<Tile>,
    connections_v: Array2D<Connection>,
    connections_h: Array2D<Connection>,
    width: usize,
    height: usize,
    connection_counter: usize,
}

impl Board {
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(x, y)
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if let Some(v) = self.tiles.get_mut(x, y) {
            *v = tile;
        }
    }

    pub fn get_connections(&self, x: usize, y: usize) -> [bool; 4] {
        let north_conn = self
            .connections_v
            .get(x, y.wrapping_sub(1))
            .unwrap_or(&DISCONNECTED_CONNECTION)
            .is_connected;
        let south_conn = self
            .connections_v
            .get(x, y)
            .unwrap_or(&DISCONNECTED_CONNECTION)
            .is_connected;
        let west_conn = self
            .connections_h
            .get(x.wrapping_sub(1), y)
            .unwrap_or(&DISCONNECTED_CONNECTION)
            .is_connected;
        let east_conn = self
            .connections_h
            .get(x, y)
            .unwrap_or(&DISCONNECTED_CONNECTION)
            .is_connected;

        [north_conn, east_conn, south_conn, west_conn]
    }

    fn get_mut_connections(&mut self, x: usize, y: usize) -> [Option<&mut Connection>; 4] {
        let (north, south) = self.connections_v.get_mut2(x, y.wrapping_sub(1), x, y);
        let (west, east) = self.connections_h.get_mut2(x.wrapping_sub(1), y, x, y);
        [north, east, south, west]
    }

    pub fn connect(&mut self, tile_x: usize, tile_y: usize, orientation: Orientation) {
        // South -> North tile_y - 1
        let connection = match orientation {
            Orientation::North => self
                .connections_v
                .get_mut(tile_x, tile_y.wrapping_sub(1))
                .expect(&format!(
                    "Tried to connect tile at position {} {} to the north",
                    tile_x, tile_y
                )),
            Orientation::South => self.connections_v.get_mut(tile_x, tile_y).expect(&format!(
                "Tried to connect tile at position {} {} to the south",
                tile_x, tile_y
            )),
            Orientation::West => self
                .connections_h
                .get_mut(tile_x.wrapping_sub(1), tile_y)
                .expect(&format!(
                    "Tried to connect tile at position {} {} to the west",
                    tile_x, tile_y
                )),
            Orientation::East => self.connections_h.get_mut(tile_x, tile_y).expect(&format!(
                "Tried to connect tile at position {} {} to the east",
                tile_x, tile_y
            )),
        };

        connection.is_connected = true;
        connection.timestamp = self.connection_counter;
        self.connection_counter += 1;

        self.update_tile(tile_x, tile_y);
        let neightbor_tile = orientation.step(tile_x, tile_y);
        self.update_tile(neightbor_tile.0, neightbor_tile.1);
    }

    fn update_tile(&mut self, x: usize, y: usize) {
        let max_conn = self.tiles.get(x, y).unwrap().max_connections();
        let mut conns = self.get_mut_connections(x, y);
        let current_conns = conns
            .iter()
            .filter(|conn| match conn {
                Some(Connection {
                    is_connected: true, ..
                }) => true,
                Some(_) => false,
                None => false,
            })
            .count();

        let need_dc = current_conns.saturating_sub(max_conn);

        if need_dc > 0 {
            conns.sort_by(|c1, c2| {
                let p1 = match c1 {
                    Some(Connection { timestamp, is_connected: false }) => *timestamp,
                    _ => usize::max_value(),
                };
                let p2 = match c2 {
                    Some(Connection { timestamp, is_connected: false }) => timestamp,
                    _ => &usize::max_value(),
                };

                p1.cmp(p2)
            });


            for conn in &mut conns[..need_dc] {
                if let Some(conn) = conn {
                    conn.is_connected = false;
                }
            }
            /*

            Sorter's 0 is North
            FlipFlop's "input" is  North

            */

            let conns = self.get_connections(x, y);
            let tile = self.tiles.get_mut(x, y).unwrap();
            let new_orientation: Option<Orientation> = match tile {
                Tile::Sorter{..} | Tile::Flipflop{..} => {

                    None
                },
                _ => {
                    None
                }
            };
        }

        // check max number of connections
        // remove Max(0, max - nconn) oldest connections
        // TODO: re-orient tile if need be
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Connection {
    /// The tick at which this connection was created
    timestamp: usize,
    is_connected: bool,
}

static DISCONNECTED_CONNECTION: Connection = Connection::disconnected();

impl Connection {
    pub const fn disconnected() -> Connection {
        Connection {
            timestamp: 0,
            is_connected: false,
        }
    }
}

impl Default for Board {
    fn default() -> Board {
        let tiles = Array2D::new(8, 8);
        Board {
            tiles,
            connections_h: Array2D::new(6, 8),
            connections_v: Array2D::new(8, 6),
            connection_counter: 0,
            width: 8,
            height: 8,
        }
    }
}

pub struct Signal {
    direction: Orientation,
}

pub struct Game {
    board: Board,
    signals: Vec<Signal>,
}

impl Game {
    pub fn with(board: Board) -> Game {
        Game {
            board,
            signals: Vec::new(),
        }
    }
}
