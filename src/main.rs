use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::from_array([150f32, 40f32]);
const PLAYER_SPEED: f32 = 700f32;

const BLOCK_SIZE: Vec2 = Vec2::from_array([100f32, 40f32]);

const BALL_SIZE: f32 = 50f32;
const BALL_SPEED: f32 = 400f32;

fn draw_title_text(text: &str, font: Font) {
    let text_font_size = 50u16;
    let text_size = measure_text(&text, Some(font), text_font_size, 1.0);
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

enum GameState {
    Menu,
    Game,
    LevelCompleted,
    Dead,
}

struct Player {
    rect: Rect,
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        let x_move = match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32,
        };

        self.rect.x += x_move * dt * PLAYER_SPEED;

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

struct Block {
    rect: Rect,
    lives: i32,
}

impl Block {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BLOCK_SIZE.x, BLOCK_SIZE.y),
            lives: 2,
        }
    }

    pub fn draw(&self) {
        let color = match self.lives {
            2 => RED,
            _ => ORANGE,
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
    return true;
}

fn reset_game(
    score: &mut i32,
    player_lives: &mut i32,
    blocks: &mut Vec<Block>,
    balls: &mut Vec<Ball>,
    player: &mut Player,
) {
    *player = Player::new();
    *score = 0;
    *player_lives = 3;
    balls.clear();
    blocks.clear();
    init_blocks(blocks);
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
        blocks.push(Block::new(board_start_pos + vec2(block_x, block_y)));
    }
}

#[macroquad::main("breakout")]
async fn main() {
    let bytes = include_bytes!("../res/Roboto-Medium.ttf");
    let mut game_state = GameState::Menu;
    let font = load_ttf_font_from_bytes(bytes).unwrap();
    let mut score = 0;
    let mut player_lives = 3;

    let mut player = Player::new();
    let mut blocks = Vec::new();
    let mut balls = Vec::new();

    init_blocks(&mut blocks);
    balls.push(Ball::new(vec2(
        screen_width() * 0.5f32,
        screen_height() * 0.7f32,
    )));

    loop {
        match game_state {
            GameState::Menu => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            }
            GameState::Game => {
                if balls.is_empty() && is_key_pressed(KeyCode::Space) {
                    balls.push(Ball::new(vec2(
                        player.rect.x + player.rect.w * 0.5f32,
                        screen_height() * 0.7f32,
                    )));
                }
                player.update(get_frame_time());
                for ball in balls.iter_mut() {
                    ball.update(get_frame_time());
                }
                for ball in balls.iter_mut() {
                    resolve_collision(&mut ball.rect, &mut ball.vel, &mut player.rect);
                    for block in blocks.iter_mut() {
                        if resolve_collision(&mut ball.rect, &mut ball.vel, &mut block.rect) {
                            block.lives -= 1;
                            if block.lives <= 0 {
                                score += 1;
                            }
                        }
                    }
                }

                let balls_len = balls.len();
                let was_last_ball = balls_len == 1;
                balls.retain(|ball| ball.rect.y < screen_height());
                let removed_balls = balls_len - balls.len();
                if removed_balls > 0 && was_last_ball {
                    player_lives -= 1;
                    if player_lives <= 0 {
                        game_state = GameState::Dead;
                    }
                }

                blocks.retain(|block| block.lives > 0);
                if blocks.is_empty() {
                    game_state = GameState::LevelCompleted;
                }
            }
            GameState::LevelCompleted | GameState::Dead => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                    reset_game(
                        &mut score,
                        &mut player_lives,
                        &mut blocks,
                        &mut balls,
                        &mut player,
                    );
                }
            }
        }

        clear_background(WHITE);
        player.draw();
        for block in blocks.iter() {
            block.draw()
        }
        for ball in balls.iter() {
            ball.draw()
        }

        match game_state {
            GameState::Menu => {
                let text = "Press SPACE to start";
                draw_title_text(text, font);
            }
            GameState::Game => {
                if balls.is_empty(){
                    let respawn_text = "Press SPACE to spawn another ball";
                    draw_title_text(respawn_text, font);
                }
                
                let score_text = format!("score : {}", score);
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
                    &format!("lives : {}", player_lives),
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
            GameState::LevelCompleted => {
                draw_title_text(&format!("You win! {} score", score), font);
            }
            GameState::Dead => {
                draw_title_text(&format!("You LOST! {} score", score), font);
            }
        }

        next_frame().await
    }
}
