#[macro_use]
mod events;
pub mod data;

use ::sdl2::render::Renderer;

struct_events! {
	keyboard: {
		key_escape: Escape,
		key_up: Up,
		key_down: Down,
		key_left: Left,
		key_right: Right,
		key_space: Space
	},
	else: {
		quit: Quit { .. }
	}
}

// Bundles the Phi abstraction in a single structure
pub struct Phi<'window> {
	pub events: Events,
	pub renderer: Renderer<'window>,
}

impl<'window> Phi<'window> {
	pub fn output_size(&self) -> (f64, f64) {
		let (w, h) = self.renderer.output_size().unwrap();
		(w as f64, h as f64)
	}
}

// Way for the currently executed view to communicate to the game loop.
pub enum ViewAction {
	None,
	Quit,
	ChangeView(Box<View>),
}

pub trait View {
	// Called on every frame
	// elapsed: expressed in seconds
	fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}

pub fn spawn<F>(title: &str, init: F)
where F: Fn(&mut Phi) -> Box<View> {
   // Init sdl2
   let sdl_context = ::sdl2::init().unwrap();
   let video = sdl_context.video().unwrap();
   let mut timer = sdl_context.timer().unwrap();	

   // Create window
   let window = video.window(title, 800, 600)
   		.position_centered().opengl().resizable()
   		.build().unwrap();

   	// Create context
   	let mut context = Phi {
   		events: Events::new(sdl_context.event_pump().unwrap()),
   		renderer: window.renderer().accelerated()
   			.build().unwrap(),
   	};

   	// Create default view
   	let mut current_view =  init(&mut context);

   	// Frame timing
   	let interval = 1_000 / 60;
   	let mut before = timer.ticks();
   	let mut last_second = timer.ticks();
   	let mut fps = 0u16;

   	loop {
   		// Frame timing (bis)
   		let now = timer.ticks();
   		let dt = now - before;
   		let elapsed = dt as f64 / 1_0000.0;

   		if dt < interval {
   			timer.delay(interval - dt);
   			continue;
   		}

   		before = now;
   		fps += 1;

   		if now - last_second > 1_000 {
   			println!("FPS: {}", fps);
   			last_second = now;
   			fps = 0;
   		}

   		// Logic and rendering
   		context.events.pump(&mut context.renderer);

   		match current_view.render(&mut context, 0.01) {
   			ViewAction::None => context.renderer.present(),
   			ViewAction::Quit => break,
   			ViewAction::ChangeView(new_view) =>
   				current_view = new_view,
   		}
   	}
}