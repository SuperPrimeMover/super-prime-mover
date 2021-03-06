use gdnative::prelude::*;
use gdnative::api::{TileMap, InputEvent, InputEventMouseMotion, InputEventMouseButton};
use gdnative::api::GlobalConstants;
use super_prime_mover::{Board, Tile, Orientation, Connection};
use bit_field::*;

#[derive(NativeClass)]
#[inherit(TileMap)]
pub struct GridTileMap {
    last_hover: Option<Vector2>,
    last_drag: Option<Vector2>,
    board: Board,
    tile_kind: TileKind
}

const CLEAR: i64 = -1;
const CABLE_W : i64 = 0;
const CABLE_E : i64 = 1;
const CABLE_H : i64 = 2;
const CABLE_NE : i64 = 3;
const CABLE_NW : i64 = 4;
const CABLE_N : i64 = 5;
const CABLE_SE : i64 = 6;
const CABLE_SW : i64 = 7;
const CABLE_S : i64 = 8;
const CABLE_UNCONNECTED : i64 = 9;
const CABLE_V: i64 = 10;
const IO_E: i64 = 11;
const IO_S: i64 = 12;
const IO_N: i64 = 13;
const IO_W: i64 = 14;
const IO_UNCONNECTED: i64 = 15;

#[methods]
impl GridTileMap {
    fn new(_owner: &Node) -> Self {
        GridTileMap {
            last_hover: None,
            last_drag: None,
            board: Board::default(),
            tile_kind: TileKind::WIRE,
        }
    }

    fn redraw_board(&self, owner: &TileMap) {
        // TODO: Use an iterator
        for y in 0..10usize {
            for x in 0..10usize {
                let tile = self.board.get_tile(x, y);
                let [n, e, s, w] = self.board.get_connections(x, y);
                let x = x as i64;
                let y = y as i64;

                const D: bool = false;
                match (tile, n, e, s, w) {
                    (Some(Tile::Empty), _, _, _, _) => {
                        owner.set_cell(x, y, CLEAR, false, false, false, Vector2::zero());
                        if let Some(v) = self.last_hover {
                            if v.x as i64 == x && v.y as i64 == y {
                                owner.set_cell(x, y, self.tile_kind.tile_num(), false, false, false, Vector2::zero());
                            }
                        }
                    },
                    (Some(Tile::Wire { .. }), D, D, D, D) => owner.set_cell(x, y, CABLE_UNCONNECTED, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), _, D, D, D) => owner.set_cell(x, y, CABLE_N, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), D, _, D, D) => owner.set_cell(x, y, CABLE_E, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), D, D, _, D) => owner.set_cell(x, y, CABLE_S, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), D, D, D, _) => owner.set_cell(x, y, CABLE_W, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), _, _, D, D) => owner.set_cell(x, y, CABLE_NE, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), _, D, _, D) => owner.set_cell(x, y, CABLE_V, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), _, D, D, _) => owner.set_cell(x, y, CABLE_NW, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), D, _, D, _) => owner.set_cell(x, y, CABLE_H, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), D, _, _, D) => owner.set_cell(x, y, CABLE_SE, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), D, D, _, _) => owner.set_cell(x, y, CABLE_SW, false, false, false, Vector2::zero()),
                    (Some(Tile::Wire { .. }), _, _, _, _) => {
                        // WTF??
                        owner.set_cell(x, y, CLEAR, false, false, false, Vector2::zero());
                        godot_print!("WEIRD: {}:{} = {:?}({},{},{},{})", x, y, tile, n, e, s, w);
                    },
                    (Some(Tile::Input { .. }), D, D, D, D) => owner.set_cell(x, y, IO_UNCONNECTED, false, false, false, Vector2::zero()),
                    (Some(Tile::Input { .. }), _, D, D, D) => owner.set_cell(x, y, IO_N, false, false, false, Vector2::zero()),
                    (Some(Tile::Input { .. }), D, _, D, D) => owner.set_cell(x, y, IO_E, false, false, false, Vector2::zero()),
                    (Some(Tile::Input { .. }), D, D, _, D) => owner.set_cell(x, y, IO_S, false, false, false, Vector2::zero()),
                    (Some(Tile::Input { .. }), D, D, D, _) => owner.set_cell(x, y, IO_W, false, false, false, Vector2::zero()),
                    (Some(Tile::Input { .. }), _, _, _, _) => {
                        // WTF??
                        owner.set_cell(x, y, CLEAR, false, false, false, Vector2::zero());
                        godot_print!("WEIRD: {}:{} = {:?}({},{},{},{})", x, y, tile, n, e, s, w);
                    },
                    _ => ()
                }
            }
        }
    }

    #[export]
    fn _input(&mut self, owner: &TileMap, event: Ref<InputEvent, Shared>) {
        if let Some(event) = event.clone().cast::<InputEventMouseMotion>() {
            let event = unsafe { event.assume_safe() };
            let local_pos = owner
                .get_global_transform()
                .inverse().unwrap()
                .transform_point(event.position().to_point())
                .to_vector();
            let tile_hover = owner.world_to_map(local_pos);
            //godot_print!("Hovering on {:?}", tile_hover);
            let tile_hover_u = if let Some(v) = tile_hover.try_cast::<usize>() {
                v
            } else { return };

            // First, remove the hover if the tile we're hovering changed
            match self.last_hover {
                Some(last_hover) if last_hover != tile_hover => {
                    owner.set_cellv(last_hover, CLEAR, false, false, false);
                    self.last_hover = None;
                }
                _ => ()
            }

            // If we're in the middle of a drag, insert a cable if necessary and
            // connect the previous drag location to the current hover
            if event.button_mask().get_bit(GlobalConstants::BUTTON_LEFT as usize - 1) {
                if let Some(&Tile::Empty) = self.board.get_tile(tile_hover_u.x, tile_hover_u.y) {
                    self.board.set_tile(tile_hover_u.x, tile_hover_u.y, Tile::Wire { slow: false });
                    owner.set_cellv(tile_hover, CABLE_UNCONNECTED, false, false, false);
                }
                self.last_drag.and_then(|v| {
                    let orientation = adjascency(v, tile_hover)?;
                    godot_print!("Connecting {:?} to {:?}", v, orientation);
                    self.board.connect(v.x as usize, v.y as usize, orientation);
                    self.redraw_board(owner);
                    Some(())
                });
                self.last_drag = Some(tile_hover);
            }

            if let Some(&Tile::Empty) = self.board.get_tile(tile_hover_u.x, tile_hover_u.y) {
                // TODO: Transparency to signal that it's just a hover.
                owner.set_cellv(tile_hover, self.tile_kind.tile_num(), false, false, false);
                self.last_hover = Some(tile_hover);
            }
        }

        // TODO: Touch screen
        if let Some(event) = event.clone().cast::<InputEventMouseButton>() {
            let event = unsafe { event.assume_safe() };
            let local_pos = owner
                .get_global_transform()
                .inverse().unwrap()
                .transform_point(event.position().to_point())
                .to_vector();
            let tile_hover = owner.world_to_map(local_pos);
            let tile_hover_u = if let Some(v) = tile_hover.try_cast::<usize>() {
                v
            } else { return };
            match event.button_index() {
                GlobalConstants::BUTTON_LEFT => {
                    self.board.set_tile(tile_hover_u.x, tile_hover_u.y, self.tile_kind.tile_data());
                    owner.set_cellv(tile_hover, self.tile_kind.tile_num(), false, false, false);
                },
                _ => (),
            }
        }
    }

    #[export]
    fn _change_tile(&mut self, owner: &TileMap, button_pressed: bool, tile_ty: u8) {
        godot_print!("Changing tile: {:?}", tile_ty);
        if tile_ty >= TileKind::MAX {
            return;
        }
        self.tile_kind = TileKind(tile_ty)
    }
}

fn adjascency(from: Vector2, to: Vector2) -> Option<Orientation> {
    let from = from.cast::<usize>();
    let to = to.cast::<usize>();
    if from.x == to.x && from.y == to.y.wrapping_sub(1) {
        Some(Orientation::South)
    } else if from.x == to.x && from.y == to.y.saturating_add(1) {
        Some(Orientation::North)
    } else if from.x == to.x.wrapping_sub(1) && from.y == to.y {
        Some(Orientation::East)
    } else if from.x == to.x.saturating_add(1) && from.y == to.y {
        Some(Orientation::West)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileKind(u8);

impl TileKind {
    const WIRE: TileKind = TileKind(0);
    const INPUT: TileKind = TileKind(1);
    const MAX: u8 = 2;

    pub fn tile_num(&self) -> i64 {
        match self {
            &TileKind::WIRE => CABLE_UNCONNECTED,
            &TileKind::INPUT => IO_UNCONNECTED,
            _ => CLEAR,
        }
    }

    pub fn tile_data(&self) -> Tile {
        match self {
            &TileKind::WIRE => Tile::Wire { slow: false },
            &TileKind::INPUT => Tile::Input { data: vec![] },
            _ => Tile::Empty,
        }
    }
}

fn init(handle: InitHandle) {
    std::env::set_var("RUST_BACKTRACE", "1");
    handle.add_class::<GridTileMap>();
}

godot_init!(init);