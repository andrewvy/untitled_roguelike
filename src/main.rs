extern crate rand;
extern crate tcod;

use std::cmp;

use rand::Rng;

use tcod::console::*;
use tcod::colors::{self, Color};
use tcod::map::{Map as FovMap, FovAlgorithm};

mod tile;
use tile::Tile;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL : Color = Color { r: 130, g: 110, b: 50 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };
const COLOR_LIGHT_GROUND : Color = Color { r: 200, g: 180, b: 50 };

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 30;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

#[derive(Debug)]
struct Object {
  x: i32,
  y: i32,
  char: char,
  color: Color,
}

impl Object {
  pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
    Object {
      x: x,
      y: y,
      char: char,
      color: color,
    }
  }

  pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map) {
    if !map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
      self.x += dx;
      self.y += dy;
    }
  }

  pub fn draw(&self, console: &mut Console) {
    console.set_default_foreground(self.color);
    console.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }

  pub fn clear(&self, console: &mut Console) {
    console.put_char(self.x, self.y, ' ', BackgroundFlag::None);
  }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
  x1: i32,
  y1: i32,
  x2: i32,
  y2: i32,
}

impl Rect {
  pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
    Rect {
      x1: x,
      y1: y,
      x2: x + w,
      y2: y + h,
    }
  }

  pub fn center(&self) -> (i32, i32) {
    let center_x = (self.x1 + self.x2) / 2;
    let center_y = (self.y1 + self.y2) / 2;
    (center_x, center_y)
  }

  pub fn intersects_with(&self, other: &Rect) -> bool {
    (self.x1 <= other.x2) && (self.x2 >= other.x1) &&
      (self.y1 <= other.y2) && (self.y2 >= other.y1)
  }
}

fn create_room(room: Rect, map: &mut Map) {
  for x in (room.x1 + 1)..room.x2 {
    for y in (room.y1 + 1)..room.y2 {
      map[x as usize][y as usize] = Tile::empty();
    }
  }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
  for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
    map[x as usize][y as usize] = Tile::empty();
  }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
  for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
    map[x as usize][y as usize] = Tile::empty();
  }
}

type Map = Vec<Vec<Tile>>;

fn make_map() -> (Map, (i32, i32)) {
  let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
  let mut rooms : Vec<Rect> = vec![];
  let mut starting_position = (0, 0);

  for _ in 0..MAX_ROOMS {
    // Generate a Width and Height of a Room.
    let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
    let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

    // Generate a random position.
    let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
    let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

    let new_room = Rect::new(x, y, w, h);
    let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));

    if !failed {
      create_room(new_room, &mut map);

      let (new_x, new_y) = new_room.center();

      if rooms.is_empty() {
        starting_position = (new_x, new_y);
      } else {
        let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

        if rand::random() {
          create_h_tunnel(prev_x, new_x, prev_y, &mut map);
          create_v_tunnel(prev_y, new_y, prev_x, &mut map);
        } else {
          create_v_tunnel(prev_y, new_y, prev_x, &mut map);
          create_h_tunnel(prev_x, new_x, prev_y, &mut map);
        }
      }

      rooms.push(new_room);
    }
  }

  (map, starting_position)
}

fn main() {
  let mut root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("Rust/libtcod tutorial")
    .renderer(Renderer::GLSL)
    .init();

  let mut con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
  let (mut map, (player_x, player_y)) = make_map();

  tcod::system::set_fps(LIMIT_FPS);

  let player = Object::new(player_x, player_y, '@', colors::WHITE);
  let npc = Object::new(player_x + 1, player_y + 2, '@', colors::YELLOW);
  let mut objects = [player, npc];

  let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      fov_map.set(x, y, !map[x as usize][y as usize].block_sight,
                        !map[x as usize][y as usize].blocked);
    }
  }

  let mut previous_player_position = (-1, -1);

  while !root.window_closed() {
    let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);

    render_all(&mut root, &mut con, &objects, &mut map, &mut fov_map, fov_recompute);
    root.flush();

    for object in &objects {
      object.clear(&mut con)
    }

    let player = &mut objects[0];
    previous_player_position = (player.x, player.y);
    let exit = handle_keys(&mut root, player, &map);

    if exit {
      break
    }
  }
}

fn render_all(root: &mut Root, console: &mut Offscreen, objects: &[Object], map: &mut Map, fov_map: &mut FovMap, fov_recompute: bool) {
  if fov_recompute {
    let player = &objects[0];
    fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);

    for y in 0..MAP_HEIGHT {
      for x in 0..MAP_WIDTH {
        let visible = fov_map.is_in_fov(x, y);
        let wall = map[x as usize][y as usize].block_sight;
        let color = match (visible, wall) {
          (false, true) => COLOR_DARK_WALL,
          (false, false) => COLOR_DARK_GROUND,

          (true, true) => COLOR_LIGHT_WALL,
          (true, false) => COLOR_LIGHT_GROUND,
        };
        let explored = &mut map[x as usize][y as usize].explored;
        if visible {
          *explored = true;
        }
        if *explored {
          console.set_char_background(x, y, color, BackgroundFlag::Set);
        }
      }
    }
  }

  for object in objects {
    if fov_map.is_in_fov(object.x, object.y) {
      object.draw(console);
    }
  }

  blit(console, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), root, (0, 0), 1.0, 1.0);
}

fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
  use tcod::input::Key;
  use tcod::input::KeyCode::*;

  let key = root.wait_for_keypress(true);

  match key {
    Key { code: Enter, alt: true, .. } => {
      let fullscreen = root.is_fullscreen();
      root.set_fullscreen(!fullscreen);
    }
    Key { code: Escape, .. } => return true,
    Key { code: Up, .. } => player.move_by(0, -1, map),
    Key { code: Down, .. } => player.move_by(0, 1, map),
    Key { code: Left, .. } => player.move_by(-1, 0, map),
    Key { code: Right, .. } => player.move_by(1, 0, map),
    _ => {},
  }

  false
}
