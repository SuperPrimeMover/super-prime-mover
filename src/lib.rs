// priorities: bas droite gauche haut

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    South,
    West,
    East,
    North,
}

#[derive(Debug, Clone)]
pub enum BoardIcon {
    Green,
    Red,
    Blue,
    Other {
        png_data: Vec<u8>
    }
}

// Signal { orientation: South }
// Wire { wire1: South, wire2: North }
//
#[derive(Debug, Clone)]
pub enum Tile {
    Empty,
    Unusable {
        /// If true, this tile is a vanilla game "broken" tile on which nothing
        /// can get placed. If false, it's an "invisible" tile that may be used
        /// as a pseudo input.
        broken: bool
    },
    Wire {
        // wire1, wire2
        slow: bool
    },
    Bridge,
    Joiner {
        orientation: Orientation
    },
    Cloner,
    Sorter {
        orientation: Orientation,
        reversed: bool
    },
    Deleter,
    Flipflop {
        orientation: Orientation,
        reversed: bool
    },
    Incrementer {
        reversed: bool
    },
    Button {
        orientation: Orientation
    },
    Lock {
        locked: bool
    },
    SubBoard {
        contents: Board,
        icon: BoardIcon
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::Empty
    }
}

impl Tile {
    pub fn max_connections(tile: Tile) -> usize {
        match tile { // oscour ?
            Tile::Button{..} => 2,
            Tile::Wire{..} => 2,
            Tile::Lock{..} => 2,
            Tile::Cloner => 3,
            Tile::Flipflop{..} => 3,
            Tile::Sorter{..} => 3,
            Tile::SubBoard{..} => 4,
            Tile::Joiner{..} => 4,
            Tile::Incrementer{..} => 4,
            Tile::Deleter => 4,
            Tile::Bridge => 4,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NDVec<T> {
    inner: Box<[T]>,
    width: usize
}

impl<T> NDVec<T> {
    pub fn new(width: usize, height: usize) -> NDVec<T>
    where
        T: Default
    {
        let mut vec = Vec::with_capacity(height * width);
        vec.resize_with(height * width, Default::default);
        NDVec {
            inner: vec.into_boxed_slice(),
            width: width,
        }
    }
    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.inner.get(y * self.width + x)
    }
    #[inline(always)]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.inner.get_mut(y * self.width + x)
    }
}

impl<'a, T> IntoIterator for &'a NDVec<T> {
    type Item = (usize, usize, &'a T);
    type IntoIter = IterNDVec<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterNDVec{ iter: (&self.inner).into_iter().enumerate(), width: self.width }
    }
}

pub struct IterNDVec<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, T>>,
    width: usize
}

impl<'a, T> Iterator for IterNDVec<'a, T> {
    type Item = (usize, usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, val) = self.iter.next()?;

        Some((idx % self.width, idx / self.width, val))
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    tiles: NDVec<Tile>,
    connections_v: NDVec<Connection>,
    connections_h: NDVec<Connection>,
    width: usize,
    height: usize,
    connection_counter: usize,
}

impl Board {
    pub fn get_connections(&self, x: usize, y: usize) -> [&Connection; 4] {
        let north_conn = self.connections_v.get(x, y.wrapping_sub(1)).unwrap_or(&DISCONNECTED_CONNECTION);
        let south_conn = self.connections_v.get(x, y).unwrap_or(&DISCONNECTED_CONNECTION);
        let west_conn = self.connections_h.get(x.wrapping_sub(1), y).unwrap_or(&DISCONNECTED_CONNECTION);
        let east_conn = self.connections_h.get(x, y).unwrap_or(&DISCONNECTED_CONNECTION);

        [north_conn, east_conn, south_conn, west_conn]
    }

    pub fn connect(&mut self, tile_x: usize, tile_y: usize, orientation: Orientation) {
        // South -> North tile_y - 1
        let connection = match orientation {
            Orientation::North => {
                self.connections_v.get_mut(tile_x, tile_y.wrapping_sub(1))
                    .expect(&format!("Tried to connect tile at position {} {} to the north", tile_x, tile_y))
            },
            Orientation::South => {
                self.connections_v.get_mut(tile_x, tile_y)
                    .expect(&format!("Tried to connect tile at position {} {} to the south", tile_x, tile_y))
            },
            Orientation::West => {
                self.connections_v.get_mut(tile_x.wrapping_sub(1), tile_y)
                    .expect(&format!("Tried to connect tile at position {} {} to the west", tile_x, tile_y))
            }
            Orientation::East => {
                self.connections_v.get_mut(tile_x, tile_y)
                    .expect(&format!("Tried to connect tile at position {} {} to the east", tile_x, tile_y))
            },
        };

        connection.is_connected = true;
        connection.timestamp = self.connection_counter;
        self.connection_counter += 1;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Connection {
    /// The tick at which this connection was created
    timestamp: usize,
    is_connected: bool
}

static DISCONNECTED_CONNECTION: Connection = Connection::disconnected();

impl Connection {
    const fn disconnected() -> Connection {
        Connection { timestamp: 0, is_connected: false }
    }
}

// board.connect(tile1, tile2)
// tile_conn1.connect(direction)


//fn get_conn(&self, x: usize, y: usize) -> [bool; 4]

impl Default for Board {
    fn default() -> Board {
        let tiles = NDVec::new(8, 8);
        Board {
            tiles,
            connections_h: NDVec::new(6, 8),
            connections_v: NDVec::new(8, 6),
            connection_counter: 0,
            width: 8,
            height: 8
        }
    }
}

pub struct Signal {
    direction: Orientation
}

pub struct Game {
    board: Board,
    signals: Vec<Signal>
}

impl Game {
    pub fn with(board: Board) {

    }
}