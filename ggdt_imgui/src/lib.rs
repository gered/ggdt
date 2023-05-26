use ggdt::graphics::RgbaBitmap;
use ggdt::system::{SystemEvent, SystemEventHandler};

use crate::platform::Platform;
use crate::renderer::Renderer;

mod platform;
mod renderer;

pub use platform::*;
pub use renderer::*;

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

	#[inline]
	pub fn texture_map(&self) -> &imgui::Textures<RgbaBitmap> {
		&self.renderer.texture_map
	}

	#[inline]
	pub fn texture_map_mut(&mut self) -> &mut imgui::Textures<RgbaBitmap> {
		&mut self.renderer.texture_map
	}

	pub fn reset_texture_map(&mut self) {
		self.renderer.reset_textures(&mut self.context);
	}
}

impl SystemEventHandler for ImGui {
	fn handle_event(&mut self, event: &SystemEvent) -> bool {
		self.platform.handle_event(&mut self.context, event)
	}
}

pub trait UiSupport {
	fn is_any_hovered(&self) -> bool;
	fn is_any_focused(&self) -> bool;
	fn image(&self, id: impl AsRef<str>, texture_id: imgui::TextureId, size: [f32; 2]);
}

impl UiSupport for imgui::Ui {
	fn is_any_hovered(&self) -> bool {
		self.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ANY_WINDOW)
	}

	fn is_any_focused(&self) -> bool {
		self.is_window_focused_with_flags(imgui::WindowFocusedFlags::ANY_WINDOW)
	}

	fn image(&self, id: impl AsRef<str>, texture_id: imgui::TextureId, size: [f32; 2]) {
		self.invisible_button(id, size);
		let draw_list = self.get_window_draw_list();
		draw_list.add_image(texture_id, self.item_rect_min(), self.item_rect_max()).build();
	}
}
