use crate::platform::Platform;
use crate::renderer::Renderer;
use ggdt::graphics::bitmap::rgb::RgbaBitmap;
use ggdt::system::event::{SystemEvent, SystemEventHandler};

pub mod platform;
pub mod renderer;

#[derive(Debug)]
pub struct ImGui {
	context: imgui::Context,
	platform: Platform,
	renderer: Renderer,
}

impl ImGui {
	pub fn new() -> Self {
		let mut context = imgui::Context::create();
		let platform = Platform::new(&mut context);
		let renderer = Renderer::new(&mut context);

		ImGui { context, platform, renderer }
	}

	pub fn new_frame(&mut self, dest: &RgbaBitmap) -> &mut imgui::Ui {
		self.platform.prepare_frame(&mut self.context, dest);
		self.context.new_frame()
	}

	pub fn render(&mut self, dest: &mut RgbaBitmap) {
		let draw_data = self.context.render();
		self.renderer.render(draw_data, dest)
	}

	#[inline]
	pub fn context(&self) -> &imgui::Context {
		&self.context
	}

	#[inline]
	pub fn context_mut(&mut self) -> &mut imgui::Context {
		&mut self.context
	}
}

impl SystemEventHandler for ImGui {
	fn handle_event(&mut self, event: &SystemEvent) -> bool {
		self.platform.handle_event(&mut self.context, event)
	}
}