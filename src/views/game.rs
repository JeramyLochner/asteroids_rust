use ::phi::{Phi, View, ViewAction};
use ::phi::data::{Rectangle, MaybeAlive};
use ::phi::gfx::{CopySprite, Sprite, AnimatedSprite, AnimatedSpriteDescr};
use ::sdl2::pixels::Color;
use ::views::shared::Background;
use ::views::bullets::*;

// Constants
const DEBUG: bool = false;

// Player Constants
const PLAYER_SPEED: f64 = 180.0;
const PLAYER_W: f64 = 43.0;
const PLAYER_H: f64 = 39.0;
const PLAYER_PATH: &'static str =  "assets/spaceship.png";
const PLAYER_MAX_LIVES: usize = 3;

// Asteroid Constants
const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;
const ASTEROID_AMOUNT: usize = 10;

//Explosion Constants
const EXPLOSION_PATH: &'static str = "assets/explosion.png";
const EXPLOSIONS_WIDE: usize = 5;
const EXPLOSIONS_HIGH: usize = 4;
const EXPLOSIONS_TOTAL: usize = 17;
const EXPLOSION_SIDE: f64 = 96.0;
const EXPLOSION_FPS: f64 = 16.0;
const EXPLOSION_DURATION: f64 = 1.0 / EXPLOSION_FPS * EXPLOSIONS_TOTAL as f64;

// The Player implementation
struct Player {
	rect: Rectangle,
	sprites: Vec<Sprite>,
	current: PlayerFrame,
	cannon: CannonType,
	lives: usize,
}

// Player's Ship's Sprite frames
#[derive(Clone, Copy)]
enum PlayerFrame {
	UpNorm = 0,
	UpFast = 1,
	UpSlow = 2,
	MidNorm = 3,
	MidFast = 4,
	MidSlow = 5,
	DownNorm = 6,
	DownFast = 7,
	DownSlow = 8
}

impl Player {
	pub fn new(phi: &mut Phi) -> Player {
		// Set up player sprites and spawn him at (64,center) with default cannon
		let spritesheet = Sprite::load(&mut phi.renderer, PLAYER_PATH).unwrap();
		let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    w: PLAYER_W,
                    h: PLAYER_H,
                    x: PLAYER_W * x as f64,
                    y: PLAYER_H * y as f64,
                }).unwrap());
            }
        }

        Player {
            rect: Rectangle {
                x: 64.0,
                y: (phi.output_size().1 - PLAYER_H) / 2.0,
                w: PLAYER_W,
                h: PLAYER_H,
            },
            sprites: sprites,
            current: PlayerFrame::MidNorm,
            cannon: CannonType::RectBullet,
            lives: PLAYER_MAX_LIVES,
        }
	}

	// Checks for weapon changes, if the player is trying to go off screen, and updates speed
	pub fn update(&mut self, phi: &mut Phi, elapsed: f64) {
		if phi.events.now.key_1 == Some(true) {
			self.cannon = CannonType::RectBullet;
		}

		if phi.events.now.key_2 == Some(true) {
			self.cannon = CannonType::SineBullet {
				amplitude: 10.0,
				angular_vel: 15.0,
			};
		}

		if phi.events.now.key_3 == Some(true) {
			self.cannon = CannonType::DivergentBullet {
				a: 100.0,
				b: 1.2,
			};			
		}

		let diagonal = 
			(phi.events.key_up ^ phi.events.key_down) &&
			(phi.events.key_left ^ phi.events.key_right);

		let moved = 
			if diagonal { 1.0 / 2.0f64.sqrt() }
			else { 1.0 } * PLAYER_SPEED * elapsed;

		let dx = match (phi.events.key_left, phi.events.key_right) {
			(true, true) | (false, false) => 0.0,
			(true, false) => -moved,
			(false, true) => moved,
		};

		let dy = match (phi.events.key_up, phi.events.key_down) {
			(true, true) | (false, false) => 0.0,
			(true, false) => -moved,
			(false, true) => moved,
		};

		self.rect.x += dx;
		self.rect.y += dy;

		let movable_region = Rectangle {
			x: 0.0,
			y: 0.0,
			w: phi.output_size().0 * 0.70,
			h: phi.output_size().1,
		};

		self.rect = self.rect.move_inside(movable_region).unwrap();

		self.current = 
			if dx == 0.0 && dy < 0.0 		{ PlayerFrame::UpNorm }
			else if dx > 0.0 && dy < 0.0 	{ PlayerFrame::UpFast }
			else if dx < 0.0 && dy < 0.0 	{ PlayerFrame::UpSlow }
			else if dx == 0.0 && dy == 0.0 	{ PlayerFrame::MidNorm }
			else if dx > 0.0 && dy == 0.0 	{ PlayerFrame::MidFast }
			else if dx < 0.0 && dy == 0.0 	{ PlayerFrame::MidSlow }
			else if dx == 0.0 && dy > 0.0 	{ PlayerFrame::DownNorm }
			else if dx > 0.0 && dy > 0.0 	{ PlayerFrame::DownFast }
			else if dx < 0.0 && dy > 0.0 	{ PlayerFrame::DownSlow }
			else { unreachable!() };
	}

	// Draw the player to the screen
	pub fn render(&self, phi: &mut Phi) {
		if DEBUG {
			phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
			phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
		}

		phi.renderer.copy_sprite(
			&self.sprites[self.current as usize],
			self.rect);
	}

	// Spawns two bullets based on cannon type on top of the player's two cannons
	pub fn spawn_bullets(&self) -> Vec<Box<Bullet>> {
		let cannons_x = self.rect.x + 30.0;
		let cannon1_y = self.rect.y + 6.0;
		let cannon2_y = self.rect.y + PLAYER_H - 10.0;
		spawn_bullets(self.cannon, cannons_x, cannon1_y, cannon2_y)
	}
}

// Asteroid Implementation
struct Asteroid {
	sprite: AnimatedSprite,
	rect: Rectangle,
	vel: f64,
}

impl Asteroid {
	// Creates the factory that will generate Asteroids
	fn factory(phi: &mut Phi) -> AsteroidFactory {
		AsteroidFactory {
			sprite: AnimatedSprite::with_fps(
				AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
					image_path: ASTEROID_PATH,
					total_frames: ASTEROIDS_TOTAL,
					frames_high: ASTEROIDS_HIGH,
					frames_wide: ASTEROIDS_WIDE,
					frame_w: ASTEROID_SIDE,
					frame_h: ASTEROID_SIDE,
				}), 1.0),
		}
	}

	// Updates location and check if offscreen
    fn update(mut self, dt: f64) -> Option<Asteroid> {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROID_SIDE {
            None
        } else {
            Some(self)
        }
    }

    // Draws asteroid to screen
	fn render(&mut self, phi: &mut Phi) {
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
        }

        phi.renderer.copy_sprite(&self.sprite, self.rect);
	}

	// returns asteroid's rectangle (x, y, w, h)
	fn rect(&self) -> Rectangle {
		self.rect
	}
}

// Asteroid Factory Implementation
struct AsteroidFactory {
	sprite: AnimatedSprite,
}

impl AsteroidFactory {
	// Selects a random y location and speed for a generated asteroid
	fn random(&self, phi: &mut Phi) -> Asteroid {
		let (w, h) = phi.output_size();

		let mut sprite = self.sprite.clone();
		sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

		Asteroid {
			sprite: sprite,
			rect: Rectangle {
				w: ASTEROID_SIDE,
				h: ASTEROID_SIDE,
				x: w,
				y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
			},
			vel: ::rand::random::<f64>().abs() * 100.0 + 50.0,
		}
	}
}

// Explosion Implementation
struct Explosion {
	sprite: AnimatedSprite,
	rect: Rectangle,

	alive_since: f64,
}

impl Explosion {
	// Returns a factory for generating Explosions on top of Asteroids
	fn factory(phi: &mut Phi) -> ExplosionFactory {
		ExplosionFactory {
			sprite: AnimatedSprite::with_fps(
				AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
					image_path: EXPLOSION_PATH,
					total_frames: EXPLOSIONS_TOTAL,
					frames_high: EXPLOSIONS_HIGH,
					frames_wide: EXPLOSIONS_WIDE,
					frame_w: EXPLOSION_SIDE,
					frame_h: EXPLOSION_SIDE,
					}), EXPLOSION_FPS),
		}
	}

	// updates the animation of the explosion
	fn update(mut self, dt: f64) -> Option<Explosion> {
		self.alive_since += dt;
		self.sprite.add_time(dt);

		if self.alive_since >= EXPLOSION_DURATION {
			None
		} else {
			Some(self)
		}
	}

	fn render(&self, phi: &mut Phi) {
		phi.renderer.copy_sprite(&self.sprite, self.rect);
	}
}

// Explosion Factory Implementation
struct ExplosionFactory {
	sprite: AnimatedSprite,
}

impl ExplosionFactory {
	// generates an explosion at the center of a given object
	fn at_center(&self, center: (f64, f64)) -> Explosion {
		let mut sprite = self.sprite.clone();

		Explosion {
			sprite: sprite,

			rect: Rectangle::with_size(EXPLOSION_SIDE, EXPLOSION_SIDE).center_at(center),

			alive_since: 0.0,
		}
	}
}


// The Game View
pub struct GameView {
	player: Player,
	bullets: Vec<Box<Bullet>>,
	asteroids: Vec<Asteroid>,
	asteroid_factory: AsteroidFactory,

	explosions: Vec<Explosion>,
	explosion_factory: ExplosionFactory,

	bg_back: Background,
	bg_middle: Background,
	bg_front: Background,
}

impl GameView {
	// Starts the game view with a new player, new generators, and the backgrounds
	pub fn new(phi: &mut Phi) -> GameView {
		GameView {
			player: Player::new(phi),

			bullets: vec![],

			asteroids: vec![],

			asteroid_factory: Asteroid::factory(phi),

			explosions: vec![],

			explosion_factory: Explosion::factory(phi),

			bg_back: Background {
				pos: 0.0,
				vel: 20.0,
				sprite: Sprite::load(&mut phi.renderer, "assets/starBG.png").unwrap(),
			},

			bg_middle: Background {
				pos: 0.0,
				vel: 40.0,
				sprite: Sprite::load(&mut phi.renderer, "assets/starMG.png").unwrap(),
			},

			bg_front: Background {
				pos: 0.0,
				vel: 80.0,
				sprite: Sprite::load(&mut phi.renderer, "assets/starFG.png").unwrap(),
			},
		}
	}
}

impl View for GameView {

	// Displays and updates every entity
	fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
		// Check if the X clicked on the screen
		if phi.events.now.quit {
			return ViewAction::Quit;
		}

		// Goes back to the main menu if esc pressed
		if phi.events.now.key_escape == Some(true) {
			return ViewAction::ChangeView(Box::new(
				::views::main_menu::MainMenuView::new(phi)));
		}

		self.player.update(phi, elapsed);

		// Update all the entities and refill the vecs with only alive entities
		let old_bullets = ::std::mem::replace(&mut self.bullets, vec![]);

		self.bullets = 
			old_bullets.into_iter()
			.filter_map(|bullet| bullet.update(phi, elapsed))
			.collect();

		self.asteroids =
			::std::mem::replace(&mut self.asteroids, vec![])
			.into_iter()
			.filter_map(|asteroid| asteroid.update(elapsed))
			.collect();

		self.explosions =
			::std::mem::replace(&mut self.explosions, vec![])
			.into_iter()
			.filter_map(|explosion| explosion.update(elapsed))
			.collect();

		//Collision Detection
		let mut player_alive = true;

		let mut transition_bullets: Vec<_> =
			::std::mem::replace(&mut self.bullets, vec![])
			.into_iter()
			.map(|bullet| MaybeAlive { alive: true, value: bullet })
			.collect();

		// Checks for asteroid collision with either a bullet or the player
		self.asteroids =
			::std::mem::replace(&mut self.asteroids, vec![])
			.into_iter()
			.filter_map(|asteroid| {
				let mut asteroid_alive = true;
				// Destroys any asteroids or bullets that collide
				for bullet in &mut transition_bullets {
					if asteroid.rect().overlaps(bullet.value.rect()) {
						asteroid_alive = false;
						bullet.alive = false;
					}
				}
				// Check for player collision
				if asteroid.rect().overlaps(self.player.rect) {
					asteroid_alive = false;
					player_alive = false;
				}

				if asteroid_alive {
					Some(asteroid)
				} else {
					self.explosions.push(
						self.explosion_factory.at_center(
							asteroid.rect().center()));
					None
				}
			})
			.collect();

		self.bullets = transition_bullets.into_iter()
			.filter_map(MaybeAlive::as_option)
			.collect();

		// Check if the player lived
		if !player_alive {
			self.player.lives-=1;
			println!("Player Lives: {}", self.player.lives);
		}

		if self.player.lives == 0 {
			return ViewAction::ChangeView(Box::new(
				::views::main_menu::MainMenuView::new(phi)));
		}

		// Check if space pressed, resulting in two bullets spawned on the player
		if phi.events.now.key_space == Some(true) {
			self.bullets.append(&mut self.player.spawn_bullets());
		}

		// Randomly create asteroids approx every % x frames
		if ::rand::random::<usize>() % 10 == 0 {
			self.asteroids.push(self.asteroid_factory.random(phi));
		}

		// Clear the screen
		phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
		phi.renderer.clear();

		// Render the backgrounds behind the entities
		self.bg_back.render(&mut phi.renderer, elapsed);
		self.bg_middle.render(&mut phi.renderer, elapsed);

		// Render all entities
		self.player.render(phi);

		for bullet in &self.bullets {
			bullet.render(phi);
		}

        for asteroid in self.asteroids.iter_mut() {
            asteroid.render(phi);
        }

        for explosion in &self.explosions {
        	explosion.render(phi);
        }

        // Render the foregrounds
		self.bg_front.render(&mut phi.renderer, elapsed);

		ViewAction::None
	}
}