#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use self::gfx::Sprite;
use ::sdl2::pixels::Color;
use ::sdl2::render::Renderer;
use ::std::collections::HashMap;
use ::std::path::Path;

// Instantiates a new event macro with keyboard/window events
struct_events! {
	keyboard: {
		key_escape: Escape,
		key_up: Up,
		key_down: Down,
		key_left: Left,
		key_right: Right,
		key_space: Space,
      key_enter: Return,

      key_1: Num1,
      key_2: Num2,
      key_3: Num3
	},
	else: {
		quit: Quit { .. }
	}
}

// Bundles the Phi abstraction in a single structure for easier parametrization
pub struct Phi<'window> {
	pub events: Events,
	pub renderer: Renderer<'window>,

   cached_fonts: HashMap<(&'static str, i32), ::sdl2_ttf::Font>,
}

impl<'window> Phi<'window> {
   fn new(events: Events, renderer: Renderer<'window>) -> Phi<'window> {
      ::sdl2_image::init(::sdl2_image::INIT_PNG);

      Phi {
         events: events,
         renderer: renderer,
         cached_fonts: HashMap::new(),
      }
   }

   // Returns the size of the window (w, h)
	pub fn output_size(&self) -> (f64, f64) {
		let (w, h) = self.renderer.output_size().unwrap();
		(w as f64, h as f64)
	}

   // Gets a string and returns a Font sprite
   pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: i32, color: Color) -> Option<Sprite> {
      
      if let Some(font) = self.cached_fonts.get(&(font_path, size)) {
         return font.render(text, ::sdl2_ttf::blended(color)).ok()
            .and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
            .map(Sprite::new)
      }

      ::sdl2_ttf::Font::from_file(Path::new(font_path), size).ok()
         .and_then(|font| { 
            self.cached_fonts.insert((font_path, size), font);
            self.ttf_str_sprite(text, font_path, size, color)
         })
   }
}

impl<'window> Drop for Phi<'window> {
   // Kills sdl2_image if the window is killed
   fn drop(&mut self) {
      ::sdl2_image::quit();
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
   let _ttf_context = ::sdl2_ttf::init();

   // Create window
   let window = video.window(title, 800, 600)
   		.position_centered().opengl().resizable()
   		.build().unwrap();

   	// Create context
   	let mut context = Phi::new(
         Events::new(sdl_context.event_pump().unwrap()),
         window.renderer().accelerated()
            .build().unwrap());

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