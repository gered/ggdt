use thiserror::Error;

use crate::graphics::{ARGBu8x4, ColorsAsBytes, IndexedBitmap, Palette, RgbaBitmap};

pub fn calculate_logical_screen_size(window_width: u32, window_height: u32, scale_factor: u32) -> (u32, u32) {
	let logical_width = (window_width as f32 / scale_factor as f32).ceil() as u32;
	let logical_height = (window_height as f32 / scale_factor as f32).ceil() as u32;
	(logical_width, logical_height)
}

const SCREEN_TEXTURE_PIXEL_SIZE: usize = 4; // 32-bit ARGB format

#[derive(Error, Debug)]
pub enum SdlFramebufferError {
	#[error("SdlFramebufferError SDL error: {0}")]
	SDLError(String),
}

/// Internal structure to manager the SDL renderer/canvas and texture bits surrounding how we display our
/// [`SystemResources`] implementation managed [`Bitmap`][Bitmap] backbuffer to the actual screen.
///
/// [Bitmap]: crate::graphics::bitmap::Bitmap
pub struct SdlFramebuffer {
	sdl_texture: sdl2::render::Texture,
	sdl_texture_pitch: usize,
	intermediate_texture: Option<Box<[ARGBu8x4]>>,
}

// TODO: i'm not totally happy with this implementation. i don't like the two display methods and how the caller
//       can technically call the wrong method. while i'm quite sure this won't happen in practice, it still feels
//       like a bad design which we could do better. but this is simple for now, so i'll leave it for the time being.
//       since this is all not in a public module, that seems like less of a big deal to me as well. for now.
impl SdlFramebuffer {
	pub fn new(
		canvas: &mut sdl2::render::WindowCanvas,
		logical_screen_width: u32,
		logical_screen_height: u32,
		create_intermediate_texture: bool,
	) -> Result<Self, SdlFramebufferError> {
		// this sets up screen/window resolution independant rendering on the SDL-side of things
		// which we may or may not actually need, but this ALSO changes the way that SDL reports things
		// like mouse cursor coordinates. so we benefit from setting the canvas logical screen size
		// to always match our window size, even when we are using variable screen sizes.
		if let Err(error) = canvas.set_logical_size(logical_screen_width, logical_screen_height) {
			return Err(SdlFramebufferError::SDLError(error.to_string()));
		}

		let format = sdl2::pixels::PixelFormatEnum::BGRA8888;

		let sdl_texture =
			match canvas.create_texture_streaming(Some(format), logical_screen_width, logical_screen_height) {
				Ok(texture) => texture,
				Err(error) => return Err(SdlFramebufferError::SDLError(error.to_string())),
			};
		let sdl_texture_pitch = sdl_texture.query().width as usize * SCREEN_TEXTURE_PIXEL_SIZE;

		let intermediate_texture = if create_intermediate_texture {
			// create a raw 32-bit RGBA buffer that will be used as the temporary source for
			// SDL texture uploads each frame. necessary as applications are dealing with 8-bit indexed
			// bitmaps, not 32-bit RGBA pixels, so this temporary buffer is where we convert the final
			// application framebuffer to 32-bit RGBA pixels before it is uploaded to the SDL texture
			let texture_pixels_size = (logical_screen_width * logical_screen_height) as usize;
			Some(vec![ARGBu8x4::default(); texture_pixels_size].into_boxed_slice())
		} else {
			None
		};

		Ok(SdlFramebuffer { sdl_texture, sdl_texture_pitch, intermediate_texture })
	}

	pub fn display_indexed_bitmap(
		&mut self,
		canvas: &mut sdl2::render::WindowCanvas,
		src: &IndexedBitmap,
		palette: &Palette,
	) -> Result<(), SdlFramebufferError> {
		let intermediate_texture = &mut self.intermediate_texture.as_mut().expect(
			"Calls to display_indexed_bitmap should only occur on SdlFramebuffers with an intermediate_texture",
		);

		src.copy_as_argb_to(intermediate_texture, palette);

		let texture_pixels = intermediate_texture.as_bytes();
		if let Err(error) = self.sdl_texture.update(None, texture_pixels, self.sdl_texture_pitch) {
			return Err(SdlFramebufferError::SDLError(error.to_string()));
		}
		canvas.clear();
		if let Err(error) = canvas.copy(&self.sdl_texture, None, None) {
			return Err(SdlFramebufferError::SDLError(error));
		}
		canvas.present();

		Ok(())
	}

	pub fn display(
		&mut self,
		canvas: &mut sdl2::render::WindowCanvas,
		src: &RgbaBitmap,
	) -> Result<(), SdlFramebufferError> {
		assert!(
			self.intermediate_texture.is_none(),
			"Calls to display should only occur on SdlFramebuffers without an intermediate_texture"
		);

		let texture_pixels = src.pixels().as_bytes();
		if let Err(error) = self.sdl_texture.update(None, texture_pixels, self.sdl_texture_pitch) {
			return Err(SdlFramebufferError::SDLError(error.to_string()));
		}
		canvas.clear();
		if let Err(error) = canvas.copy(&self.sdl_texture, None, None) {
			return Err(SdlFramebufferError::SDLError(error));
		}
		canvas.present();

		Ok(())
	}
}
