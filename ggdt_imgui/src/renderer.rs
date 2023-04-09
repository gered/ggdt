use ggdt::graphics::bitmap::rgb::{RgbaBitmap, RgbaPixelFormat};
use ggdt::graphics::color::{to_argb32, BlendFunction};
use ggdt::math::rect::Rect;
use ggdt::math::vector2::Vector2;
use imgui::internal::RawWrapper;

#[derive(Debug)]
pub struct Renderer {
	pub texture_map: imgui::Textures<RgbaBitmap>,
}

impl Renderer {
	pub fn new(context: &mut imgui::Context) -> Self {
		let mut texture_map = imgui::Textures::new();

		// set up a bitmap with the imgui font atlas texture pixels and register a bitmap->texture mapping for it
		// with imgui
		let mut font = context.fonts();
		let mut font_atlas_texture = font.build_rgba32_texture();
		font.tex_id = texture_map.insert(
			RgbaBitmap::from_bytes(
				font_atlas_texture.width,
				font_atlas_texture.height,
				RgbaPixelFormat::RGBA,
				&mut font_atlas_texture.data,
			)
			.unwrap(),
		);

		Renderer { texture_map }
	}

	pub fn render(&mut self, draw_data: &imgui::DrawData, dest: &mut RgbaBitmap) {
		let original_clip_rect = *dest.clip_region();

		for draw_list in draw_data.draw_lists() {
			for command in draw_list.commands() {
				match command {
					imgui::DrawCmd::Elements { count, cmd_params } => {
						let clip_rect = Rect::from_coords(
							(cmd_params.clip_rect[0] - draw_data.display_pos[0]) as i32,
							(cmd_params.clip_rect[1] - draw_data.display_pos[1]) as i32,
							(cmd_params.clip_rect[2] - draw_data.display_pos[0]) as i32,
							(cmd_params.clip_rect[3] - draw_data.display_pos[1]) as i32,
						);
						if !clip_rect.overlaps(&dest.full_bounds()) {
							continue;
						}

						dest.set_clip_region(&clip_rect);
						let bitmap = self.texture_map.get(cmd_params.texture_id).unwrap();

						let idx_buffer = draw_list.idx_buffer();
						let vtx_buffer = draw_list.vtx_buffer();
						for idx in (0..count).step_by(3) {
							let v1 = vtx_buffer[idx_buffer[cmd_params.idx_offset + idx] as usize];
							let v2 = vtx_buffer[idx_buffer[cmd_params.idx_offset + idx + 1] as usize];
							let v3 = vtx_buffer[idx_buffer[cmd_params.idx_offset + idx + 2] as usize];

							dest.solid_textured_multicolor_blended_triangle_2d(
								&[
									Vector2::new(v2.pos[0], v2.pos[1]),
									Vector2::new(v1.pos[0], v1.pos[1]),
									Vector2::new(v3.pos[0], v3.pos[1]),
								],
								&[
									Vector2::new(v2.uv[0], v2.uv[1]),
									Vector2::new(v1.uv[0], v1.uv[1]),
									Vector2::new(v3.uv[0], v3.uv[1]),
								],
								&[
									to_argb32(v2.col[3], v2.col[0], v2.col[1], v2.col[2]),
									to_argb32(v1.col[3], v1.col[0], v1.col[1], v1.col[2]),
									to_argb32(v3.col[3], v3.col[0], v3.col[1], v3.col[2]),
								],
								bitmap,
								BlendFunction::Blend,
							);
						}
					}
					imgui::DrawCmd::RawCallback { callback, raw_cmd } => unsafe { callback(draw_list.raw(), raw_cmd) },
					imgui::DrawCmd::ResetRenderState => {
						dest.set_clip_region(&original_clip_rect);
					}
				}
			}
		}
		dest.set_clip_region(&original_clip_rect);
	}
}
