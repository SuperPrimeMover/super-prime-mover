use gdnative::prelude::*;
use gdnative::api::{TileMap, InputEvent, InputEventMouseMotion};

#[derive(NativeClass)]
#[inherit(TileMap)]
pub struct GridTileMap {
    current_selection: Vector2,
}

const CLEAR: i64 = -1;
const W_CABLE: i64 = 0;

#[methods]
impl GridTileMap {
    fn new(_owner: &Node) -> Self {
        GridTileMap {
            current_selection: Vector2::zero()
        }
    }

    #[export]
    fn _input(&mut self, owner: &TileMap, event: Ref<InputEvent, Shared>) {
        if let Some(event) = event.cast::<InputEventMouseMotion>() {
            let event = unsafe { event.assume_safe() };
            let tile_hover = owner.world_to_map(event.as_ref().position());
            if tile_hover != self.current_selection {
                owner.set_cellv(self.current_selection, CLEAR, false, false, false);
                owner.set_cellv(tile_hover, W_CABLE, false, false, false);
                self.current_selection = tile_hover;
            }
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<GridTileMap>();
}

godot_init!(init);