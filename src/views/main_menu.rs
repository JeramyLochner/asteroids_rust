use ::phi::{Phi, View, ViewAction};
use ::phi::data::Rectangle;
use ::phi::gfx::{CopySprite, Sprite};
use ::sdl2::pixels::Color;
use ::views::shared::Background;

struct Action {

	// Function which is executed when action chosen
	func: Box<Fn(&mut Phi) -> ViewAction>,

	// Sprite is rendered when the player does not focus on this option
	idle_sprite: Sprite,

	// Sprite is rendered when the player focuses on this option
	hover_sprite: Sprite,
}

impl Action {
	fn new(phi: &mut Phi, label: &'static str, func: Box<Fn(&mut Phi) -> ViewAction>) -> Action {
		Action {
			func: func,
			idle_sprite: phi.ttf_str_sprite(label, "assets/belligerent.ttf", 32, Color::RGB(220, 220, 200)).unwrap(),
			hover_sprite: phi.ttf_str_sprite(label, "assets/belligerent.ttf", 38, Color::RGB(255, 255, 255)).unwrap(),
		}
	}
}

pub struct MainMenuView {
	actions: Vec<Action>,
	selected: i8,

	bg_back: Background,
	bg_middle: Background,
	bg_front: Background,
}

impl MainMenuView {
	pub fn new(phi: &mut Phi) -> MainMenuView {
		MainMenuView {
			actions: vec![
				Action::new(phi, "New Game", Box::new(|phi| {
					ViewAction::ChangeView(Box::new(::views::game::GameView::new(phi)))
				})),
				Action::new(phi, "Quit", Box::new(|_| {
					ViewAction::Quit
				})),
			],

			selected: 0,

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

impl View for MainMenuView {
	fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
		if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
			return ViewAction::Quit;
		}

		if phi.events.now.key_space == Some(true) {
			return (self.actions[self.selected as usize].func)(phi);
		}

		if phi.events.now.key_up == Some(true) {
			self.selected -= 1;
			if self.selected < 0 {
				self.selected = self.actions.len() as i8 - 1;
			}
		}

		if phi.events.now.key_down == Some(true) {
			self.selected += 1;
			if self.selected >= self.actions.len() as i8 {
				self.selected = 0;
			}
		}

		phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
		phi.renderer.clear();

		self.bg_back.render(&mut phi.renderer, elapsed);
		self.bg_middle.render(&mut phi.renderer, elapsed);
		self.bg_front.render(&mut phi.renderer, elapsed);

		let (win_w, win_h) = phi.output_size();
		let label_h = 50.0;
		let border_width = 3.0;
		let box_w = 360.0;
		let box_h = self.actions.len() as f64 * label_h;
		let margin_h = 10.0;

		phi.renderer.set_draw_color(Color::RGB(70, 15, 70));
		phi.renderer.fill_rect(Rectangle {
			w: box_w + border_width * 2.0,
			h: box_h + border_width * 2.0 + margin_h * 2.0,
			x: (win_w - box_w) / 2.0 - border_width,
			y: (win_h - box_h) / 2.0 - margin_h - border_width,
		}.to_sdl().unwrap());

		phi.renderer.set_draw_color(Color::RGB(140, 30, 140));
		phi.renderer.fill_rect(Rectangle {
			w: box_w,
			h: box_h + margin_h * 2.0,
			x: (win_w - box_w) / 2.0,
			y: (win_h - box_h) / 2.0 - margin_h,
		}.to_sdl().unwrap());

		for (i, action) in self.actions.iter().enumerate() {
			if self.selected as usize == i {
				let (w, h) = action.hover_sprite.size();
				phi.renderer.copy_sprite(&action.hover_sprite, Rectangle {
					x: (win_w - w) / 2.0,
					y: (win_h - box_h + label_h - h) / 2.0 + label_h * i as f64,
					w: w,
					h: h,
				});
			} else {
				let (w, h) = action.idle_sprite.size();
                phi.renderer.copy_sprite(&action.idle_sprite, Rectangle {
                    x: (win_w - w) / 2.0,
                    y: (win_h - box_h + label_h - h) / 2.0 + label_h * i as f64,
                    w: w,
                    h: h,
                });
			}
		}

		ViewAction::None
	}
}