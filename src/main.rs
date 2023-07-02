use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SQUARE_SIZE: usize = 40;
const BULLET_SPEED: f32 = 5.0;

const MAX_ENEMIES: usize = 10;

struct Square {
    x: f32,
    y: f32,
    angle: f32,
}

impl Square {
    fn new(x: f32, y: f32) -> Square {
        Square {
            x,
            y,
            angle: 0.0,
        }
    }

    fn rotate_left(&mut self) {
        self.angle -= 0.1;
    }

    fn rotate_right(&mut self) {
        self.angle += 0.1;
    }

    fn fire_bullet(&self, target_x: f32, target_y: f32) -> Bullet {
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let angle = dy.atan2(dx);

        let velocity_x = angle.cos() * BULLET_SPEED;
        let velocity_y = angle.sin() * BULLET_SPEED;

        Bullet::new(self.x, self.y, velocity_x, velocity_y)
    }
}

struct Bullet {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
}

impl Bullet {
    fn new(x: f32, y: f32, velocity_x: f32, velocity_y: f32) -> Bullet {
        Bullet {
            x,
            y,
            velocity_x,
            velocity_y,
        }
    }

    fn update(&mut self) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;
    }

    fn is_out_of_bounds(&self) -> bool {
        self.x < 0.0 || self.x >= WIDTH as f32 || self.y < 0.0 || self.y >= HEIGHT as f32
    }
}

struct Enemy {
    x: f32,
    y: f32,
}

impl Enemy {
    fn new(x: f32, y: f32) -> Enemy {
        Enemy { x, y }
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Interactive Window",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut square = Square::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut enemies: Vec<Enemy> = Vec::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::W) {
            square.y -= 1.0;
        }
        if window.is_key_down(Key::A) {
            square.x -= 1.0;
        }
        if window.is_key_down(Key::S) {
            square.y += 1.0;
        }
        if window.is_key_down(Key::D) {
            square.x += 1.0;
        }
        if window.is_key_down(Key::Left) {
            square.rotate_left();
        }
        if window.is_key_down(Key::Right) {
            square.rotate_right();
        }
        if window.get_mouse_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = match window.get_mouse_pos(MouseMode::Clamp) {
                Some((x, y)) => (x, y),
                None => (0.0, 0.0), // Default position when mouse position is not available
            };
            bullets.push(square.fire_bullet(mouse_x, mouse_y));
        }

        if rand::thread_rng().gen_range(0..100) < 5 && enemies.len() < MAX_ENEMIES {
            let enemy_x = rand::thread_rng().gen_range(0..WIDTH) as f32;
            let enemy_y = rand::thread_rng().gen_range(0..HEIGHT) as f32;
            enemies.push(Enemy::new(enemy_x, enemy_y));
        }

        let mut bullets_to_remove: Vec<usize> = Vec::new();
        for (bullet_index, bullet) in bullets.iter_mut().enumerate() {
            bullet.update();
            if bullet.is_out_of_bounds() {
                bullets_to_remove.push(bullet_index);
            } else {
                for (enemy_index, enemy) in enemies.iter().enumerate() {
                    if is_collision(bullet.x, bullet.y, enemy.x, enemy.y) {
                        bullets_to_remove.push(bullet_index);
                        enemies.remove(enemy_index);
                        break;
                    }
                }
            }
        }

        // Remove bullets that went out of bounds or hit an enemy
        for &bullet_index in bullets_to_remove.iter().rev() {
            bullets.remove(bullet_index);
        }

        bullets_to_remove.clear();

        buffer.iter_mut().for_each(|pixel| *pixel = 0x00_00_00); // Clear the buffer

        draw_square(&mut buffer, square.x as i32, square.y as i32, square.angle);
        for bullet in bullets.iter() {
            draw_bullet(&mut buffer, bullet.x as i32, bullet.y as i32);
        }
        for enemy in enemies.iter() {
            draw_enemy(&mut buffer, enemy.x as i32, enemy.y as i32);
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}

fn draw_square(buffer: &mut [u32], x: i32, y: i32, angle: f32) {
    let square_color = 0xFF_00_00; // Red color
    let half_size = SQUARE_SIZE as i32 / 2;
    for i in -half_size..half_size {
        for j in -half_size..half_size {
            let rotated_x = (i as f32 * angle.cos() - j as f32 * angle.sin()) as i32;
            let rotated_y = (i as f32 * angle.sin() + j as f32 * angle.cos()) as i32;
            let buffer_x = x + rotated_x;
            let buffer_y = y + rotated_y;
            if buffer_x >= 0 && buffer_x < WIDTH as i32 && buffer_y >= 0 && buffer_y < HEIGHT as i32 {
                let index = (buffer_x as usize) + (buffer_y as usize) * WIDTH;
                buffer[index] = square_color;
            }
        }
    }
}

fn draw_bullet(buffer: &mut [u32], x: i32, y: i32) {
    let bullet_color = 0x00_FF_00; // Green color
    let bullet_size = 4;
    let half_size = bullet_size / 2;
    for i in -half_size..half_size {
        for j in -half_size..half_size {
            let buffer_x = x + i;
            let buffer_y = y + j;
            if buffer_x >= 0 && buffer_x < WIDTH as i32 && buffer_y >= 0 && buffer_y < HEIGHT as i32 {
                let index = (buffer_x as usize) + (buffer_y as usize) * WIDTH;
                buffer[index] = bullet_color;
            }
        }
    }
}

fn draw_enemy(buffer: &mut [u32], x: i32, y: i32) {
    let enemy_color = 0x00_00_FF; // Blue color
    let enemy_size = 20;
    let half_size = enemy_size / 2;
    for i in -half_size..half_size {
        for j in -half_size..half_size {
            let buffer_x = x + i;
            let buffer_y = y + j;
            if buffer_x >= 0 && buffer_x < WIDTH as i32 && buffer_y >= 0 && buffer_y < HEIGHT as i32 {
                let index = (buffer_x as usize) + (buffer_y as usize) * WIDTH;
                buffer[index] = enemy_color;
            }
        }
    }
}

fn is_collision(x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    distance < SQUARE_SIZE as f32 / 2.0
}
