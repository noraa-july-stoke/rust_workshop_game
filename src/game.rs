#![allow(dead_code)]
use piston_window::*;
use rand::random;
use std::collections::HashSet;

const PLAYER_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const ENEMY_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BULLET_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const PLAYER_SPEED: f64 = 200.0; //Pixels per second
const BULLET_SPEED: f64 = 300.0;
const ENEMY_SPEED: f64 = 50.0;
const PLAYER_SPEED_BOOSTED: f64 = 245.0; // in pixels per second
const BOOST_TIME: f64 = 30.0; // in seconds
const POWERUP_SPEED: f64 = 140.0; // in pixels per second
const POWERUP_SPAWN_TIME: f64 = 20.0; // in seconds
const POWERUP_COLOR: [f32; 4] = [0.5, 0.0, 0.5, 1.0]; // purple
const POWERUP_CHANCE: f32 = 0.5; // 50% chance

const MAX_ENEMIES_ON_GROUND: i32 = 10;

#[derive(Clone, PartialEq)]
struct Entity {
    x: f64,
    y: f64,
}

#[derive(Clone, PartialEq)]
enum PowerUpType {
    SpeedBoost,
    TripleShot,
}

struct PowerUp {
    x: f64,
    y: f64,
    power_up_type: PowerUpType,
}

pub struct Game {
    player: Entity,
    bullets: Vec<Entity>,
    enemies: Vec<Entity>,
    enemy_spawn_timer: f64,
    window_width: f64,
    window_height: f64,
    key_state: Option<Key>,
    power_ups: Vec<PowerUp>,
    power_up_spawn_timer: f64,
    power_up_active: Option<PowerUpType>,
    power_up_active_timer: f64,
}

impl Game {
    pub fn new(window_width: f64, window_height: f64) -> Game {
        Game {
            player: Entity {
                x: window_width / 2.0,
                y: window_height - 20.0,
            },
            bullets: Vec::new(),
            enemies: Vec::new(),
            enemy_spawn_timer: 0.0,
            window_width,
            window_height,
            key_state: None,
            power_ups: Vec::new(),
            power_up_spawn_timer: 0.0,
            power_up_active: None,
            power_up_active_timer: 0.0,
        }
    }

    pub fn key_pressed(&mut self, key: Key) {
        self.key_state = Some(key);
        if let Some(Key::Space) = self.key_state {
            self.bullets.push(Entity {
                x: self.player.x,
                y: self.player.y - 20.0,
            })
        }
    }

    pub fn key_released(&mut self, _key: Key) {
        self.key_state = None;
    }

    pub fn update(&mut self, dt: f64) {
        // key pressed?
        match self.key_state {
            Some(Key::Left) => self.player.x -= PLAYER_SPEED * dt,
            Some(Key::Right) => self.player.x += PLAYER_SPEED * dt,
            _ => (),
        }

        // spawn enemies
        self.enemy_spawn_timer += dt;
        if self.enemy_spawn_timer > 1.0 {
            self.enemies.push(Entity {
                x: random::<f64>() * self.window_width,
                y: 0.0,
            });
            self.enemy_spawn_timer = 0.0;
        }

        // update bullets
        for bullet in &mut self.bullets {
            bullet.y -= BULLET_SPEED * dt;
        }
        // update enemies
        for enemy in &mut self.enemies {
            enemy.y += ENEMY_SPEED * dt;
        }

        // spawn power_ups
        self.power_up_spawn_timer += dt;
        if self.power_up_spawn_timer > POWERUP_SPAWN_TIME {
            let power_up_type = if random::<f32>() < POWERUP_CHANCE {
                PowerUpType::TripleShot
            } else {
                PowerUpType::SpeedBoost
            };

            self.power_ups.push(PowerUp {
                x: random::<f64>() * self.window_width,
                y: 0.0,
                power_up_type,
            });
            self.power_up_spawn_timer = 0.0;
        }

        // update power_up
        for power_up in &mut self.power_ups {
            power_up.y += POWERUP_SPEED * dt
        }

        // check power-up player collisions

        if let Some(power_up_index) = self.power_ups.iter().position(|power_up| {
            let dx = power_up.x - self.player.x;
            let dy = power_up.y - self.player.y;
            (dx * dy + dy * dy).sqrt() < 30.0
        }) {
            let power_up = self.power_ups.remove(power_up_index);
            self.power_up_active = Some(power_up.power_up_type);
            self.power_up_active_timer = BOOST_TIME;
        }

        // Handle active power-up effects
        if let Some(power_up_type) = &self.power_up_active {
            match power_up_type {
                PowerUpType::SpeedBoost => {
                    // The player's speed will be boosted when moving
                    match self.key_state {
                        Some(Key::Left) => self.player.x -= PLAYER_SPEED_BOOSTED * dt,
                        Some(Key::Right) => self.player.x += PLAYER_SPEED_BOOSTED * dt,
                        _ => (),
                    }
                }
                PowerUpType::TripleShot => {
                    // The player will shoot three bullets when shooting
                    if let Some(Key::Space) = self.key_state {
                        self.bullets.push(Entity {
                            x: self.player.x - 10.0,
                            y: self.player.y - 20.0,
                        });
                        self.bullets.push(Entity {
                            x: self.player.x,
                            y: self.player.y - 20.0,
                        });
                        self.bullets.push(Entity {
                            x: self.player.x + 10.0,
                            y: self.player.y - 20.0,
                        });
                        self.key_state = None;
                    }
                }
            }
            // Decrease the active power-up timer
            self.power_up_active_timer -= dt;
            if self.power_up_active_timer <= 0.0 {
                self.power_up_active = None;
            }
        }

        // remove off-screen power-up
        self.power_ups
            .retain(|power_up: &PowerUp| power_up.y < self.window_height);

        let enemies_on_ground = self
            .enemies
            .iter()
            .filter(|e| e.y >= self.window_height - 20.0)
            .count();

        if enemies_on_ground as i32 >= MAX_ENEMIES_ON_GROUND {
            self.reset();
        }

        // update bullets
        for bullet in &mut self.bullets {
            bullet.y -= BULLET_SPEED * dt;
        }

        // remove off-screen enemies
        let mut enemies_on_ground = 0;
        self.enemies.retain(|enemy| {
            if enemy.y >= self.window_height - 20.0 {
                enemies_on_ground += 1;
            }
            enemy.y < self.window_height
        });

        // check bullet-enemy collisions
        let mut bullet_indices_to_remove = HashSet::new();
        let mut enemy_indices_to_remove = HashSet::new();

        for (b_index, bullet) in self.bullets.iter().enumerate() {
            for (e_index, enemy) in self.enemies.iter().enumerate() {
                let dx = bullet.x - enemy.x;
                let dy = bullet.y - enemy.y;
                if (dx * dx + dy * dy).sqrt() < 10.0 {
                    bullet_indices_to_remove.insert(b_index);
                    enemy_indices_to_remove.insert(e_index);
                }
            }
        }

        let bullets_clone = self.bullets.clone();
        self.bullets = bullets_clone
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !bullet_indices_to_remove.contains(index))
            .map(|(_, item)| item)
            .collect();

        let enemies_clone = self.enemies.clone();
        self.enemies = enemies_clone
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !enemy_indices_to_remove.contains(index))
            .map(|(_, item)| item)
            .collect();
    }

    pub fn draw(&self, c: &Context, g: &mut G2d) {
        clear([0.0, 0.0, 0.0, 0.1], g);

        // draw player

        let (player_x, player_y) = (self.player.x, self.player.y);
        rectangle(
            PLAYER_COLOR,
            [player_x, player_y - 10.0, 20.0, 20.0],
            c.transform,
            g,
        );
        rectangle(
            PLAYER_COLOR,
            [player_x - 20.0, player_y - 10.0, 20.0, 20.0],
            c.transform,
            g,
        );
        // draw bullets
        for bullet in &self.bullets {
            let (bullet_x, bullet_y) = (bullet.x, bullet.y);
            rectangle(
                BULLET_COLOR,
                [bullet_x - 2.0, bullet_y - 10.0, 4.0, 20.0],
                c.transform,
                g,
            );
        }

        // draw enemies
        for enemy in &self.enemies {
            let (enemy_x, enemy_y) = (enemy.x, enemy.y);
            rectangle(
                ENEMY_COLOR,
                [enemy_x - 10.0, enemy_y - 10.0, 20.0, 20.0],
                c.transform,
                g,
            );
        }

        // draw power-ups
        for power_up in &self.power_ups {
            let (power_up_x, power_up_y) = (power_up.x, power_up.y);
            rectangle(
                POWERUP_COLOR,
                [power_up_x - 10.0, power_up_y - 10.0, 20.0, 20.0],
                c.transform,
                g,
            );
            rectangle(
                POWERUP_COLOR,
                [power_up_x + 10.0, power_up_y - 10.0, 20.0, 20.0],
                c.transform,
                g,
            );
        }
    }

    pub fn reset(&mut self) {
        self.player = Entity {
            x: self.window_width / 2.0,
            y: self.window_height - 20.0,
        };
        self.bullets.clear();
        self.enemies.clear();
        self.enemy_spawn_timer = 0.0;
    }
}

pub fn main() {
    let (width, height) = (800, 800);
    let mut window: PistonWindow = WindowSettings::new("Space Invaders", (width, height))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game::new(width as f64, height as f64);

    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            game.key_pressed(key);
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            game.key_released(key);
        }

        if let Some(update_args) = e.update_args() {
            game.update(update_args.dt);
        }

        window.draw_2d(&e, |c, g, _| {
            game.draw(&c, g);
        });
    }
}
