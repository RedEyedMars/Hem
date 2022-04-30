pub mod b;
pub mod benny;
pub mod e;
pub mod input;
pub mod socket;

use b::Board;
use e::a::Particle;
use e::b::AbilityClass;
use e::m::Monster;
use e::p::Player;

//use crate::packed_simd::f32x4;
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum EntityType {
    Wall,
    Player,
    Monster,
    Particle,
    Nothing,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Direction {
    East,
    West,
    North,
    South,
}

use packed_simd::{f32x2, f32x4, u32x2};
const VEL: f32 = 0.1f32;
impl Direction {
    pub fn advance(&self, xy: f32x2, width: f32, height: f32) -> Result<f32x2, ()> {
        use Direction::*;
        match self {
            East if xy.extract(0) + VEL < width => Ok(xy + f32x2::new(VEL, 0f32)),
            West if xy.extract(0) > 0f32 => Ok(xy - f32x2::new(VEL, 0f32)),
            North if xy.extract(1) + VEL < height => Ok(xy + f32x2::new(0f32, VEL)),
            South if xy.extract(1) > 0f32 => Ok(xy - f32x2::new(0f32, VEL)),
            _ => Err(()),
        }
    }
    pub fn retreat(&self, xy: u32x2, width: u32, height: u32) -> Result<u32x2, ()> {
        use Direction::*;
        match self {
            East if xy.extract(0) > 0 => Ok(xy - u32x2::new(1u32, 032)),
            West if xy.extract(0) + 1 < width => Ok(xy + u32x2::new(1u32, 032)),
            South if xy.extract(1) + 1 < height => Ok(xy + u32x2::new(0u32, 132)),
            North if xy.extract(1) < height => Ok(xy - u32x2::new(0u32, 132)),
            _ => Err(()),
        }
    }
    pub fn turn_dex(self) -> Direction {
        use Direction::*;
        match self {
            East => South,
            South => West,
            West => North,
            North => East,
        }
    }
    pub fn turn_sin(self) -> Direction {
        use Direction::*;
        match self {
            East => North,
            South => East,
            West => South,
            North => West,
        }
    }

    pub fn from_varying(c: char) -> Direction {
        match c {
            'N' => Direction::North,
            'E' => Direction::East,
            'W' => Direction::West,
            'S' => Direction::South,
            _ => Direction::East,
        }
    }
    pub fn to_varying(&self) -> char {
        match self {
            Direction::North => 'N',
            Direction::East => 'E',
            Direction::West => 'W',
            Direction::South => 'S',
        }
    }
    pub fn vel(&self) -> f32x2 {
        match self {
            Direction::West => f32x2::new(-1f32, 0.0f32),
            Direction::East => f32x2::new(1f32, 0.0f32),
            Direction::North => f32x2::new(0f32, 1f32),
            Direction::South => f32x2::new(0f32, -1f32),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Colour {
    Red,    // +
    Green,  // x
    Blue,   // ^
    Purple, // v
}

impl Colour {
    pub fn from_varying(c: char) -> Colour {
        match c {
            'P' => Colour::Purple,
            'R' => Colour::Red,
            'B' => Colour::Blue,
            'G' => Colour::Green,
            _ => Colour::Purple,
        }
    }
    pub fn to_varying(&self) -> char {
        match self {
            Colour::Blue => 'B',
            Colour::Red => 'R',
            Colour::Green => 'G',
            Colour::Purple => 'P',
        }
    }

    pub fn from_optional_varying(c: char) -> Option<Colour> {
        match c {
            'P' => Some(Colour::Purple),
            'R' => Some(Colour::Red),
            'B' => Some(Colour::Blue),
            'G' => Some(Colour::Green),
            'X' => None,
            _ => None,
        }
    }
    pub fn to_optional_varying(colour: &Option<Colour>) -> char {
        match colour {
            Some(Colour::Blue) => 'B',
            Some(Colour::Red) => 'R',
            Some(Colour::Green) => 'G',
            Some(Colour::Purple) => 'P',
            None => 'X',
        }
    }
}

pub enum Collider<'a> {
    Enemy(&'a Board, &'a mut Vec<Monster>, &'a Vec<Player>),
    HomeField(&'a Board, &'a mut Vec<Monster>, &'a Player),
}
impl<'a> Collider<'a> {
    pub fn has_collision(&self, bx: f32x4) -> bool {
        match self {
            Collider::Enemy(board, _, _) => Collider::has_board_collision(board, &bx),
            Collider::HomeField(board, _, _) => Collider::has_board_collision(board, &bx),
        }
    }
    fn has_board_collision(board: &Board, bx: &f32x4) -> bool {
        let tile_xy = f32x2::new(0.1f32, 0.1f32);
        let tile_wh = f32x2::new(0.8f32, 0.8f32);
        let box_xy = f32x2::new(bx.extract(0), bx.extract(1));
        let box_wh = f32x2::new(bx.extract(2), bx.extract(3));
        board
            .walls()
            .iter()
            .map(|(_, xy)| *xy)
            .filter(|xy| (*xy + tile_wh).gt(box_xy).all())
            .any(|xy| box_wh.gt(xy + tile_xy).all())
    }
}

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub struct GameState {
    //pub clock: Instant,
    board: Board,
    boards: Vec<Board>,
    player: Player,
    enemy_players: Vec<Player>,
    monsters: Vec<Monster>,
    particles: Vec<Particle>,
    board_index: usize,
    frames_to_compute: f32,
}

static mut META: [i32; 32] = [0; 32];

const WASM_MEMORY_BUFFER_SIZE: usize = 1024;
static mut IMAGE_IDS: [i32; WASM_MEMORY_BUFFER_SIZE] = [0; WASM_MEMORY_BUFFER_SIZE];
static mut XS: [f32; WASM_MEMORY_BUFFER_SIZE] = [0f32; WASM_MEMORY_BUFFER_SIZE];
static mut YS: [f32; WASM_MEMORY_BUFFER_SIZE] = [0f32; WASM_MEMORY_BUFFER_SIZE];

#[wasm_bindgen]
pub fn meta() -> *const i32 {
    unsafe { META.as_ptr() }
}

#[wasm_bindgen]
pub fn img_ids() -> *const i32 {
    unsafe { IMAGE_IDS.as_ptr() }
}

#[wasm_bindgen]
pub fn xs() -> *const f32 {
    unsafe { XS.as_ptr() }
}

#[wasm_bindgen]
pub fn ys() -> *const f32 {
    unsafe { YS.as_ptr() }
}

use web_sys::WebSocket;
#[wasm_bindgen]
impl GameState {
    pub async fn new() -> GameState {
        let mut game = GameState {
            //clock: Instant::now(),
            board: b::Board::empty(),
            board_index: 0usize,
            boards: vec![],
            player: Player::new(8f32, 8f32),
            enemy_players: vec![],
            monsters: vec![],
            particles: vec![],
            frames_to_compute: 0f32,
        };
        crate::s::l::read_level("test_b0", &mut game)
            .await
            .expect("Found error");
        game.board.render_text();
        game
    }

    pub fn set_frames_to_compute(&mut self, frames: f32) {
        self.frames_to_compute += frames;
    }
    pub fn run(&mut self) -> bool {
        self.execute().expect("Could not run game without crashing")
    }
    fn execute(&mut self) -> Result<bool, failure::Error> {
        if input::detect_input(self)? {
            return Ok(false);
        }
        if self.frames_to_compute > 1f32 {
            for _ in 0..(self.frames_to_compute as u32) {
                self.board.execute()?;
                let colliders =
                    &Collider::Enemy(&self.board, &mut self.monsters, &self.enemy_players);
                self.player.execute(colliders);
                let colliders = &Collider::HomeField(&self.board, &mut self.monsters, &self.player);
                for p in self.enemy_players.iter_mut() {
                    //p.execute(colliders);
                }
            }
            self.frames_to_compute -= (self.frames_to_compute as u32) as f32;
            socket::write_to_sockets(vec![self.player.get_socket_message()]);
            socket::read_from_sockets(self);
            crate::s::l::save_level("test_b1", &self);
        }

        //
        Ok(true)
    }
    pub fn render(&self) {
        let mut i_count = 0;
        let (player_i, player_x, player_y) = self.player.render();
        for y in 0..self.board.height() {
            for x in 0..self.board.width() {
                for i in self.board.get_tile(x as u32, y as u32).render() {
                    unsafe {
                        IMAGE_IDS[i_count] = i;
                        XS[i_count] = x as f32 - (player_x-3f32) as f32;
                        YS[i_count] = y as f32 - (player_y-3f32) as f32;
                    }
                    i_count += 1;
                }
            }
        }
        for (_i, _m) in self.monsters.iter().enumerate() {
            //m.render();
            i_count += 1;
        }
        for p in self.enemy_players.iter() {
            let (i, x, y) = p.render();
            unsafe {
                IMAGE_IDS[i_count] = i;
                XS[i_count] = x;
                YS[i_count] = y;
            }
            i_count += 1;
        }
        unsafe {
            IMAGE_IDS[i_count] = player_i;
            XS[i_count] = 3f32;
            YS[i_count] = 3f32;
        }
        i_count += 1;
        for (_i, _p) in self.particles.iter().enumerate() {
            //p.render();
            i_count += 1;
        }
        unsafe {
            META[0] = i_count as i32;
        }
    }
}

impl GameState {
    pub fn setup_boards(&mut self) {
        self.boards = Vec::with_capacity(512)
    }
    pub fn set_first_board(&mut self) {
        self.board_index = 0usize;
        self.board = self.boards.get(self.board_index).unwrap().clone();
    }
    pub fn push_board(&mut self, board: Board) {
        self.boards.push(board);
    }
    pub fn get_mut_board(&mut self, i: usize) -> &mut Board {
        self.boards.get_mut(i).expect("Could not get board")
    }
    pub fn boards(&self) -> &Vec<Board> {
        &self.boards
    }
    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn move_player(&mut self, d: Direction) {
        self.player.push(d);
    }
    pub fn activate_player(&mut self, ability: AbilityClass) {
        self.player.set_next_ability(ability);
    }
    pub fn handle_enemy_player(&mut self, remote_index: u8, player_msg: socket::SocketMessage) {
        if remote_index >= self.enemy_players.len() as u8 {
            self.enemy_players.push(Player::new_enemy());
        }
        self.enemy_players
            .get_mut(remote_index as usize)
            .unwrap()
            .accept_state(player_msg);
    }
}
