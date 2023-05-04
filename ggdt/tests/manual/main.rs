//! These are tests that need to be manually run and interacted with by the user in order to verify functionality.
//! This is specifically for integration-style testing which is testing things like windowing, system events, input
//! devices, etc that are more difficult to test in a purely automated way.
//!
//! TODO: Is there a better way to do this, rather than marking all of these tests as ignored? We don't want them to
//!       run during any automatic CI step, etc, as those should always run without manual user intervention midway.
//!
//! HACK: It is not currenly possible to run all of these together via cargo CLI, using something like
//!       `cargo test -- --test-threads=1 --ignored`. Even restricting to 1 thread like this will fail. This appears
//!       to be due to a bug in rust-sdl2 where `sdl2::sdl::IS_MAIN_THREAD_DECLARED` is never reset to false after
//!       `SDL_Quit` is finally called (which only can happen when all subsystems are dropped). Attempting to restart
//!       SDL even after a supposedly clean shutdown will always result in the error
//!       "Cannot initialize `Sdl` from more than once thread."
//!       However, you can run them via an alternative test runner like nextest (https://nexte.st/).
//!       e.g.`cargo nextest run -j 1 --run-ignored ignored-only`

mod keyboard;
mod mouse;
mod system_events;
mod system_resources_doslike;
mod system_resources_standard;

use ggdt::prelude::*;

const BACKGROUND_COLOR: ARGBu8x4 = ARGBu8x4::from_rgb([0x2c, 0x30, 0x41]);

fn draw_base_screen<BitmapType>(
	dest: &mut BitmapType,
	bg_color_1: BitmapType::PixelType,
	bg_color_2: BitmapType::PixelType,
	color: BitmapType::PixelType,
	highlight_color: BitmapType::PixelType,
) where
	BitmapType: GeneralBitmap,
{
	for y in 0..dest.height() as i32 {
		dest.horiz_line(0, dest.right() as i32, y, if y % 2 == 0 { bg_color_1 } else { bg_color_2 });
	}

	dest.horiz_line(0, 16, 0, color);
	dest.horiz_line(0, 16, dest.bottom() as i32, color);
	dest.horiz_line(dest.right() as i32 - 16, dest.right() as i32, 0, color);
	dest.horiz_line(dest.right() as i32 - 16, dest.right() as i32, dest.bottom() as i32, color);
	dest.vert_line(0, 0, 16, color);
	dest.vert_line(dest.right() as i32, 0, 16, color);
	dest.vert_line(0, dest.bottom() as i32 - 16, dest.bottom() as i32, color);
	dest.vert_line(dest.right() as i32, dest.bottom() as i32 - 16, dest.bottom() as i32, color);

	dest.set_pixel(0, 0, highlight_color);
	dest.set_pixel(0, dest.bottom() as i32, highlight_color);
	dest.set_pixel(dest.right() as i32, 0, highlight_color);
	dest.set_pixel(dest.right() as i32, dest.bottom() as i32, highlight_color);
}
