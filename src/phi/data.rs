use ::sdl2::rect::Rect as SdlRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
	pub x: f64,
	pub y: f64,
	pub w: f64,
	pub h: f64,
}

impl Rectangle {
	// Generates an SDL-compatible Rect
	pub fn to_sdl(self) -> Option<SdlRect> {
		assert!(self.w >= 0.0 && self.h >= 0.0);

		SdlRect::new(self.x as i32, self.y as i32, self.w as u32, self.h as u32).unwrap()
	}

	pub fn move_inside(self, parent: Rectangle) -> Option<Rectangle> {
		if self.w > parent.w || self.h > parent.h {
			return None;
		}

		Some(Rectangle {
			w: self.w,
			h: self.h,
			x: if self.x < parent.x { parent.x }
				else if self.x + self.w >= parent.x + parent.w { parent.x + parent.w - self.w }
				else {self.x },
			y: if self.y < parent.y { parent.y }
				else if self.y + self.h >= parent.y + parent.h { parent.y + parent.h - self.h }
				else {self.y},
		})
	}

	pub fn contains(&self, rect: Rectangle) -> bool {
        let xmin = rect.x;
        let xmax = xmin + rect.w as f64;
        let ymin = rect.y;
        let ymax = ymin + rect.h as f64;

        xmin >= self.x && xmin <= self.x + self.w as f64 &&
        xmax >= self.x && xmax <= self.x + self.w as f64 &&
        ymin >= self.y && ymin <= self.y + self.h as f64 &&
        ymax >= self.y && ymax <= self.y + self.h as f64
	}

	pub fn overlaps(&self, other: Rectangle) -> bool {
		self.x < other.x + other.w as f64 &&
		self.x + self.w as f64 > other.x &&
		self.y < other.y + other.h as f64 &&
		self.y + self.h as f64 > other.y
	}

	pub fn with_size(w: f64, h: f64) -> Rectangle {
		Rectangle {
			w: w,
			h: h,
			x: 0.0,
			y: 0.0,
		}
	}

	pub fn center_at(self, center: (f64, f64)) -> Rectangle {
		Rectangle {
			x: center.0 - self.w / 2.0,
			y: center.1 - self.h / 2.0,
			..self
		}
	}

	pub fn center(self) -> (f64, f64) {
		let x = self.x + self.w / 2.0;
		let y = self.y + self.h / 2.0;

		(x, y)
	}
}

pub struct MaybeAlive<T> {
	pub alive: bool,
	pub value: T,
}

impl<T> MaybeAlive<T> {
	pub fn as_option(self) -> Option<T> {
		if self.alive {
			Some(self.value)
		} else {
			None
		}
	}
}