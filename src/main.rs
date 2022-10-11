use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::from_array([150f32, 40f32]);
const PLAYER_INCREASED_SIZE: f32 = PLAYER_SIZE.x + 50f32;
const PLAYER_INITIAL_SPEED: f32 = 700f32;
const PLAYER_SPEED_POWERUP: f32 = 1000f32;

const BLOCK_SIZE: Vec2 = Vec2::from_array([100f32, 40f32]);

const BALL_SIZE: f32 = 50f32;
const BALL_SPEED: f32 = 400f32;

fn draw_title_text(text: &str, font: Font) {
    let text_font_size = 50u16;
    let text_size = measure_text(text, Some(font), text_font_size, 1.0);
    draw_text_ex(
        text,
        screen_width() * 0.5f32 - text_size.width * 0.5f32,
        screen_height() * 0.5f32 - text_size.height * 0.5f32,
        TextParams {
            font,
            font_size: text_font_size,
            color: BLACK,
            ..Default::default()
        },
    );
}

enum GamePlayState {
    Menu,
    Game,
    LevelCompleted,
    Dead,
}

struct Player {
    rect: Rect,
    speed : f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5f32 - PLAYER_SIZE.x * 0.5f32,
                screen_height() - 100f32,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            ),
            speed : PLAYER_INITIAL_SPEED
        }
    }

    pub fn update(&mut self, dt: f32) {
        let x_move = match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32,
        };

        self.rect.x += x_move * dt * self.speed;

        let left_side: f32 = 0f32;
        if self.rect.x < left_side {
            self.rect.x = left_side;
        }
        let right_side: f32 = screen_width() - self.rect.w;
        if self.rect.x > right_side {
            self.rect.x = right_side;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLUE);
    }
}

enum BlockType {
    Regular,
    SpawnBallOnDeath,
    SizeIncrease,
    SpeedIncrease,
}

struct Block {
    rect: Rect,
    lives: i32,
    block_type: BlockType,
}

impl Block {
    pub fn new(pos: Vec2, block_type: BlockType) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BLOCK_SIZE.x, BLOCK_SIZE.y),
            lives: 2,
            block_type,
        }
    }

    pub fn draw(&self) {
        let color = match self.block_type {
            BlockType::Regular => match self.lives {
                2 => RED,
                _ => ORANGE,
            },
            BlockType::SpawnBallOnDeath => match self.lives {
                2 => DARKGREEN,
                _ => GREEN,
            },
            BlockType::SizeIncrease => match self.lives {
                2 => DARKBLUE,
                _ => BLUE,
            },
            BlockType::SpeedIncrease => match self.lives {
                2 => PURPLE,
                _ => PINK,
            },
        };
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
    }
}

struct Ball {
    rect: Rect,
    vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BALL_SIZE, BALL_SIZE),
            vel: vec2(rand::gen_range(-1f32, 1f32), 1f32).normalize(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.rect.x += self.vel.x * dt * BALL_SPEED;
        self.rect.y += self.vel.y * dt * BALL_SPEED;

        let left_side = 0f32;
        if self.rect.x < left_side {
            self.vel.x = 1f32;
        }
        let right_side = screen_width() - self.rect.w;
        if self.rect.x > right_side {
            self.vel.x = -1f32;
        }
        let top = 0f32;
        if self.rect.y < top {
            self.vel.y = 1f32;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, DARKGRAY);
    }
}

fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    let intersection = match a.intersect(*b) {
        Some(intersection) => intersection,
        None => return false,
    };

    let a_center = a.center();
    let b_center = b.center();
    let to = b_center - a_center;
    let to_signum = to.signum();
    match intersection.w > intersection.h {
        true => {
            a.y -= to_signum.y * intersection.h;
            vel.y = -to_signum.y * vel.y.abs();
        }
        false => {
            a.x -= to_signum.x * intersection.h;
            vel.x = -to_signum.x * vel.x.abs();
        }
    }
    true
}

struct PowerupTimer {
    time_left :f32 ,
}

impl PowerupTimer {
    fn new() -> Self{
        Self {
            time_left : 0f32,
        }
    }

    fn start_timer(&mut self, length : f32){
        self.time_left = length;
    }

    fn update(&mut self, dt: f32) {
        self.time_left -= dt;
    }

    fn is_timer_done(&self) -> bool {
        if self.time_left <= 0f32 {
            return true;
        }
        false
    }
}

struct GameState {
    score: i32,
    player_lives: i32,
    blocks: Vec<Block>,
    balls: Vec<Ball>,
    player: Player,
    increase_size_timer : PowerupTimer,
    increase_speed_timer : PowerupTimer
}

impl GameState {
    fn new() -> Self {
        GameState {
            score: 0,
            player_lives: 3,
            balls: Vec::new(),
            blocks: {
                let mut blocks = Vec::new();
                init_blocks(&mut blocks);
                blocks
            },
            player: Player::new(),
            increase_size_timer : PowerupTimer::new(),
            increase_speed_timer : PowerupTimer::new(),
        }
    }
}

fn init_blocks(blocks: &mut Vec<Block>) {
    let (width, height) = (6, 6);
    let padding = 5f32;
    let total_block_size = BLOCK_SIZE + vec2(padding, padding);
    let board_start_pos = vec2(
        (screen_width() - (total_block_size.x * width as f32)) * 0.5f32,
        50f32,
    );
    for i in 0..width * height {
        let block_x = (i % width) as f32 * total_block_size.x;
        let block_y = (i / width) as f32 * total_block_size.y;
        blocks.push(Block::new(
            board_start_pos + vec2(block_x, block_y),
            BlockType::Regular,
        ));
    }
    for _ in 0..3 {
        let rand_index = rand::gen_range(0, blocks.len());
        blocks[rand_index].block_type = BlockType::SpawnBallOnDeath;
    }
    for _ in 0..3 {
        let rand_index = rand::gen_range(0, blocks.len());
        blocks[rand_index].block_type = BlockType::SizeIncrease;
    }
    for _ in 0..3 {
        let rand_index = rand::gen_range(0, blocks.len());
        blocks[rand_index].block_type = BlockType::SpeedIncrease;
    }
}

#[macroquad::main("breakout")]
async fn main() {
    let bytes = include_bytes!("../res/Roboto-Medium.ttf");
    let mut game_state = GamePlayState::Menu;
    let font = load_ttf_font_from_bytes(bytes).unwrap();

    let mut game_run_state = GameState::new();

    game_run_state.balls.push(Ball::new(vec2(
        screen_width() * 0.5f32,
        screen_height() * 0.7f32,
    )));

    loop {
        match game_state {
            GamePlayState::Menu => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GamePlayState::Game;
                }
            }
            GamePlayState::Game => {
                game_run_state.player.update(get_frame_time());
                for ball in game_run_state.balls.iter_mut() {
                    ball.update(get_frame_time());
                }
                game_run_state.increase_size_timer.update(get_frame_time());
                if game_run_state.increase_size_timer.is_timer_done(){
                    game_run_state.player.rect.w = PLAYER_SIZE.x;
                }
                if game_run_state.increase_speed_timer.is_timer_done(){
                    game_run_state.player.speed = PLAYER_INITIAL_SPEED;
                }

                let mut spawn_later = vec![];
                for ball in game_run_state.balls.iter_mut() {
                    resolve_collision(&mut ball.rect, &mut ball.vel, &game_run_state.player.rect);
                    for block in game_run_state.blocks.iter_mut() {
                        if resolve_collision(&mut ball.rect, &mut ball.vel, &block.rect) {
                            block.lives -= 1;
                            if block.lives <= 0 {
                                game_run_state.score += 1;
                                if let BlockType::SpawnBallOnDeath = block.block_type {
                                    spawn_later.push(Ball::new(ball.rect.point()));
                                }
                                if let BlockType::SizeIncrease = block.block_type {
                                    game_run_state.increase_size_timer.start_timer(10f32);
                                    game_run_state.player.rect.w = PLAYER_INCREASED_SIZE;
                                }
                                if let BlockType::SpeedIncrease = block.block_type {
                                    game_run_state.increase_speed_timer.start_timer(10f32);
                                    game_run_state.player.speed = PLAYER_SPEED_POWERUP;
                                }
                            }
                        }
                    }
                }
                for ball in spawn_later.into_iter() {
                    game_run_state.balls.push(ball);
                }

                let balls_len = game_run_state.balls.len();
                game_run_state
                    .balls
                    .retain(|ball| ball.rect.y < screen_height());
                let removed_balls = balls_len - game_run_state.balls.len();
                if removed_balls > 0 && game_run_state.balls.is_empty() {
                    game_run_state.player_lives -= 1;
                    if game_run_state.player_lives <= 0 {
                        game_state = GamePlayState::Dead;
                        continue;
                    }
                    game_run_state.balls.push(Ball::new(
                        game_run_state.player.rect.point() + vec2(0f32, -50f32),
                    ));
                }

                game_run_state.blocks.retain(|block| block.lives > 0);
                if game_run_state.blocks.is_empty() {
                    game_state = GamePlayState::LevelCompleted;
                }
            }
            GamePlayState::LevelCompleted | GamePlayState::Dead => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GamePlayState::Menu;
                    game_run_state = GameState::new();
                }
            }
        }

        clear_background(WHITE);
        game_run_state.player.draw();
        for block in game_run_state.blocks.iter() {
            block.draw()
        }
        for ball in game_run_state.balls.iter() {
            ball.draw()
        }

        match game_state {
            GamePlayState::Menu => {
                let text = "Press SPACE to start";
                draw_title_text(text, font);
            }
            GamePlayState::Game => {
                let score_text = format!("score : {}", game_run_state.score);
                let score_text_font_size = 30u16;
                let score_text_size =
                    measure_text(&score_text, Some(font), score_text_font_size, 1.0);
                draw_text_ex(
                    &score_text,
                    screen_width() * 0.5f32 - score_text_size.width * 0.5f32,
                    40.0,
                    TextParams {
                        font,
                        font_size: score_text_font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                draw_text_ex(
                    &format!("lives : {}", game_run_state.player_lives),
                    30.0,
                    40.0,
                    TextParams {
                        font,
                        font_size: score_text_font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );
            }
            GamePlayState::LevelCompleted => {
                draw_title_text(&format!("You win! {} score", game_run_state.score), font);
            }
            GamePlayState::Dead => {
                draw_title_text(&format!("You LOST! {} score", game_run_state.score), font);
            }
        }

        next_frame().await
    }
}
