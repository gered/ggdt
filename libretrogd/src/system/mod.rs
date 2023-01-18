use std::fmt::Formatter;

use byte_slice_cast::AsByteSlice;
use sdl2::{AudioSubsystem, EventPump, Sdl, TimerSubsystem, VideoSubsystem};
use sdl2::audio::AudioSpecDesired;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use thiserror::Error;

use crate::{DEFAULT_SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::audio::*;
use crate::graphics::*;

pub use self::event::*;
pub use self::input_devices::*;
pub use self::input_devices::keyboard::*;
pub use self::input_devices::mouse::*;

pub mod event;
pub mod input_devices;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("System init error: {0}")]
    InitError(String),

    #[error("System display error: {0}")]
    DisplayError(String),

    #[error("System audio error: {0}")]
    AudioError(#[from] crate::audio::AudioError),
}

/// Builder for configuring and constructing an instance of [`System`].
#[derive(Debug)]
pub struct SystemBuilder {
    window_title: String,
    vsync: bool,
    target_framerate: Option<u32>,
    initial_scale_factor: u32,
    resizable: bool,
    show_mouse: bool,
    relative_mouse_scaling: bool,
    integer_scaling: bool,
}

impl SystemBuilder {
    /// Returns a new [`SystemBuilder`] with a default configuration.
    pub fn new() -> SystemBuilder {
        SystemBuilder {
            window_title: String::new(),
            vsync: false,
            target_framerate: None,
            initial_scale_factor: DEFAULT_SCALE_FACTOR,
            resizable: true,
            show_mouse: false,
            relative_mouse_scaling: true,
            integer_scaling: false,
        }
    }

    /// Set the window title for the [`System`] to be built.
    pub fn window_title(&mut self, window_title: &str) -> &mut SystemBuilder {
        self.window_title = window_title.to_string();
        self
    }

    /// Enables or disables V-Sync for the [`System`] to be built. Enabling V-sync automatically
    /// disables `target_framerate`.
    pub fn vsync(&mut self, enable: bool) -> &mut SystemBuilder {
        self.vsync = enable;
        self.target_framerate = None;
        self
    }

    /// Sets a target framerate for the [`System`] being built to run at. This is intended to be
    /// used when V-sync is not desired, so setting a target framerate automatically disables
    /// `vsync`.
    pub fn target_framerate(&mut self, target_framerate: u32) -> &mut SystemBuilder {
        self.target_framerate = Some(target_framerate);
        self.vsync = false;
        self
    }

    /// Sets an integer scaling factor for the [`System`] being built to up-scale the virtual
    /// framebuffer to when displaying it on screen.
    pub fn scale_factor(&mut self, scale_factor: u32) -> &mut SystemBuilder {
        self.initial_scale_factor = scale_factor;
        self
    }

    /// Sets whether the window will be resizable by the user for the [`System`] being built.
    pub fn resizable(&mut self, enable: bool) -> &mut SystemBuilder {
        self.resizable = enable;
        self
    }

    /// Enables or disables mouse cursor display by the operating system when the cursor is over
    /// the window for the [`System`] being built. Disable this if you intend to render your own
    /// custom mouse cursor.
    pub fn show_mouse(&mut self, enable: bool) -> &mut SystemBuilder {
        self.show_mouse = enable;
        self
    }

    /// Enables or disables automatic DPI scaling of mouse relative movement values (delta values)
    /// available via the [`Mouse`] input device.
    pub fn relative_mouse_scaling(&mut self, enable: bool) -> &mut SystemBuilder {
        self.relative_mouse_scaling = enable;
        self
    }

    /// Enables or disables restricting the final rendered output to always be integer scaled,
    /// even if that result will not fully fill the area of the window.
    pub fn integer_scaling(&mut self, enable: bool) -> &mut SystemBuilder {
        self.integer_scaling = enable;
        self
    }

    /// Builds and returns a [`System`] from the current configuration.
    pub fn build(&self) -> Result<System, SystemError> {
        // todo: maybe let this be customized in the future, or at least halved so a 160x120 mode can be available ... ?
        let screen_width = SCREEN_WIDTH;
        let screen_height = SCREEN_HEIGHT;
        let texture_pixel_size = 4; // 32-bit ARGB format

        sdl2::hint::set(
            "SDL_MOUSE_RELATIVE_SCALING",
            if self.relative_mouse_scaling {
                "1"
            } else {
                "0"
            },
        );

        // build all the individual SDL subsystems

        let sdl_context = match sdl2::init() {
            Ok(sdl_context) => sdl_context,
            Err(message) => return Err(SystemError::InitError(message)),
        };

        let sdl_timer_subsystem = match sdl_context.timer() {
            Ok(timer_subsystem) => timer_subsystem,
            Err(message) => return Err(SystemError::InitError(message)),
        };

        let sdl_video_subsystem = match sdl_context.video() {
            Ok(video_subsystem) => video_subsystem,
            Err(message) => return Err(SystemError::InitError(message)),
        };

        let sdl_event_pump = match sdl_context.event_pump() {
            Ok(event_pump) => event_pump,
            Err(message) => return Err(SystemError::InitError(message)),
        };

        let sdl_audio_subsystem = match sdl_context.audio() {
            Ok(audio_subsystem) => audio_subsystem,
            Err(message) => return Err(SystemError::InitError(message)),
        };

        // create the window

        let window_width = screen_width * self.initial_scale_factor;
        let window_height = screen_height * self.initial_scale_factor;
        let mut window_builder = &mut (sdl_video_subsystem.window(
            self.window_title.as_str(),
            window_width,
            window_height,
        ));
        if self.resizable {
            window_builder = window_builder.resizable();
        }
        let sdl_window = match window_builder.build() {
            Ok(window) => window,
            Err(error) => return Err(SystemError::InitError(error.to_string())),
        };

        sdl_context.mouse().show_cursor(self.show_mouse);

        // turn the window into a canvas (under the hood, an SDL Renderer that owns the window)

        let mut canvas_builder = sdl_window.into_canvas();
        if self.vsync {
            canvas_builder = canvas_builder.present_vsync();
        }
        let mut sdl_canvas = match canvas_builder.build() {
            Ok(canvas) => canvas,
            Err(error) => return Err(SystemError::InitError(error.to_string())),
        };
        if let Err(error) = sdl_canvas.set_logical_size(screen_width, screen_height) {
            return Err(SystemError::InitError(error.to_string()));
        };

        // TODO: newer versions of rust-sdl2 support this directly off the WindowCanvas struct
        unsafe {
            sdl2::sys::SDL_RenderSetIntegerScale(
                sdl_canvas.raw(),
                if self.integer_scaling {
                    sdl2::sys::SDL_bool::SDL_TRUE
                } else {
                    sdl2::sys::SDL_bool::SDL_FALSE
                },
            );
        }

        // create an SDL texture which we will be uploading to every frame to display the
        // application's framebuffer

        let sdl_texture = match sdl_canvas.create_texture_streaming(
            Some(PixelFormatEnum::ARGB8888),
            screen_width,
            screen_height,
        ) {
            Ok(texture) => texture,
            Err(error) => return Err(SystemError::InitError(error.to_string())),
        };
        let sdl_texture_pitch = (sdl_texture.query().width * texture_pixel_size) as usize;

        // create a raw 32-bit RGBA buffer that will be used as the temporary source for
        // SDL texture uploads each frame. necessary as applications are dealing with 8-bit indexed
        // bitmaps, not 32-bit RGBA pixels, so this temporary buffer is where we convert the final
        // application framebuffer to 32-bit RGBA pixels before it is uploaded to the SDL texture
        let texture_pixels_size = (screen_width * screen_height * texture_pixel_size) as usize;
        let texture_pixels = vec![0u32; texture_pixels_size].into_boxed_slice();

        // create the Bitmap object that will be exposed to the application acting as the system
        // backbuffer

        let framebuffer = match Bitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT) {
            Ok(bmp) => bmp,
            Err(error) => return Err(SystemError::InitError(error.to_string())),
        };

        // create the default palette, initialized to the VGA default palette. also exposed to the
        // application for manipulation

        let palette = match Palette::new_vga_palette() {
            Ok(palette) => palette,
            Err(error) => return Err(SystemError::InitError(error.to_string())),
        };

        // create the default font, initialized to the VGA BIOS default font.

        let font = match BitmaskFont::new_vga_font() {
            Ok(font) => font,
            Err(error) => return Err(SystemError::InitError(error.to_string())),
        };

        let audio_spec = AudioSpecDesired {
            freq: Some(TARGET_AUDIO_FREQUENCY as i32),
            channels: Some(TARGET_AUDIO_CHANNELS),
            samples: None,
        };
        let mut audio = Audio::new(audio_spec, &sdl_audio_subsystem)?;
        audio.resume();
        let audio_queue = AudioQueue::new(&audio);

        // create input device objects, exposed to the application

        let keyboard = Keyboard::new();
        let mouse = Mouse::new();

        Ok(System {
            sdl_context,
            sdl_audio_subsystem,
            sdl_video_subsystem,
            sdl_timer_subsystem,
            sdl_canvas,
            sdl_texture,
            sdl_texture_pitch,
            sdl_event_pump,
            texture_pixels,
            audio,
            audio_queue,
            video: framebuffer,
            palette,
            font,
            keyboard,
            mouse,
            target_framerate: self.target_framerate,
            target_framerate_delta: None,
            next_tick: 0,
        })
    }
}

/// Holds all primary structures necessary for interacting with the operating system and for
/// applications to render to the display, react to input device events, etc. through the
/// "virtual machine" exposed by this library.
#[allow(dead_code)]
pub struct System {
    sdl_context: Sdl,
    sdl_audio_subsystem: AudioSubsystem,
    sdl_video_subsystem: VideoSubsystem,
    sdl_timer_subsystem: TimerSubsystem,
    sdl_canvas: WindowCanvas,
    sdl_texture: Texture,
    sdl_texture_pitch: usize,
    sdl_event_pump: EventPump,

    texture_pixels: Box<[u32]>,

    target_framerate: Option<u32>,
    target_framerate_delta: Option<i64>,
    next_tick: i64,

    /// An [`Audio`] instance that allows interacting with the system's audio output device.
    pub audio: Audio,

    /// An [`AudioQueue`] instance that can queue up playback/stop commands to be issued to the
    /// system's [`Audio`] instance a bit more flexibly. If you use this, your application must
    /// manually call [`AudioQueue::apply`] or [`AudioQueue::apply_to_device`] in your loop to
    /// flush the queued commands, otherwise this queue will not do anything.
    pub audio_queue: AudioQueue,

    /// The primary backbuffer [`Bitmap`] that will be rendered to the screen whenever
    /// [`System::display`] is called. Regardless of the actual window size, this bitmap is always
    /// [`SCREEN_WIDTH`]x[`SCREEN_HEIGHT`] pixels in size.
    pub video: Bitmap,

    /// The [`Palette`] that will be used in conjunction with the `video` backbuffer to
    /// render the final output to the screen whenever [`System::display`] is called.
    pub palette: Palette,

    /// A pre-loaded [`Font`] that can be used for text rendering.
    pub font: BitmaskFont,

    /// The current keyboard state. To ensure it is updated each frame, you should call
    /// [`System::do_events`] or [`System::do_events_with`] each frame.
    pub keyboard: Keyboard,

    /// The current mouse state. To ensure it is updated each frame, you should call
    /// [`System::do_events`] or [`System::do_events_with`] each frame.
    pub mouse: Mouse,
}

impl std::fmt::Debug for System {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("audio", &self.audio)
            .field("audio_queue", &self.audio_queue)
            .field("video", &self.video)
            .field("palette", &self.palette)
            .field("font", &self.font)
            .field("keyboard", &self.keyboard)
            .field("mouse", &self.mouse)
            .field("target_framerate", &self.target_framerate)
            .field("target_framerate_delta", &self.target_framerate_delta)
            .field("next_tick", &self.next_tick)
            .finish_non_exhaustive()
    }
}

impl System {
    /// Takes the `video` backbuffer bitmap and `palette` and renders it to the window, up-scaled
    /// to fill the window (preserving aspect ratio of course). If V-sync is enabled, this method
    /// will block to wait for V-sync. Otherwise, if a target framerate was configured a delay
    /// might be used to try to meet that framerate.
    pub fn display(&mut self) -> Result<(), SystemError> {
        self.mouse.render_cursor(&mut self.video);

        // convert application framebuffer to 32-bit RGBA pixels, and then upload it to the SDL
        // texture so it will be displayed on screen

        self.video
            .copy_as_argb_to(&mut self.texture_pixels, &self.palette);

        let texture_pixels = self.texture_pixels.as_byte_slice();
        if let Err(error) = self
            .sdl_texture
            .update(None, texture_pixels, self.sdl_texture_pitch)
        {
            return Err(SystemError::DisplayError(error.to_string()));
        }
        self.sdl_canvas.clear();
        if let Err(error) = self.sdl_canvas.copy(&self.sdl_texture, None, None) {
            return Err(SystemError::DisplayError(error));
        }
        self.sdl_canvas.present();

        self.mouse.hide_cursor(&mut self.video);

        // if a specific target framerate is desired, apply some loop timing/delay to achieve it
        // TODO: do this better. delaying when running faster like this is a poor way to do this..

        if let Some(target_framerate) = self.target_framerate {
            if self.target_framerate_delta.is_some() {
                // normal path for every other loop iteration except the first
                let delay = self.next_tick - self.ticks() as i64;
                if delay < 0 {
                    // this loop iteration took too long, no need to delay
                    self.next_tick -= delay;
                } else {
                    // this loop iteration completed before next_tick time, delay by the remainder
                    // time period so we're running at about the desired framerate
                    self.delay(((delay * 1000) / self.tick_frequency() as i64) as u32);
                }
            } else {
                // this branch will occur on the first main loop iteration. we use the fact that
                // target_framerate_delta was not yet set to avoid doing any delay on the first
                // loop, just in case there was some other processing between the System struct
                // being created and the actual beginning of the first loop ...
                self.target_framerate_delta =
                    Some((self.tick_frequency() / target_framerate as u64) as i64);
            }

            // expected time for the next display() call to happen by
            self.next_tick = (self.ticks() as i64) + self.target_framerate_delta.unwrap();
        }

        Ok(())
    }

    /// Checks for and responds to all SDL2 events waiting in the queue. Each event is passed to
    /// all [`InputDevice`]'s automatically to ensure input device state is up to date.
    pub fn do_events(&mut self) {
        self.do_events_with(|_event| {});
    }

    /// Same as [`System::do_events`] but also takes a function which will be called for each
    /// SDL2 event being processed (after everything else has already processed it), allowing
    /// your application to also react to any events received.
    pub fn do_events_with<F>(&mut self, mut f: F)
    where
        F: FnMut(&SystemEvent),
    {
        self.keyboard.update();
        self.mouse.update();
        self.sdl_event_pump.pump_events();
        for event in self.sdl_event_pump.poll_iter() {
            if let Ok(event) = event.try_into() {
                self.keyboard.handle_event(&event);
                self.mouse.handle_event(&event);
                f(&event);
            }
        }
    }

    pub fn ticks(&self) -> u64 {
        self.sdl_timer_subsystem.performance_counter()
    }

    pub fn tick_frequency(&self) -> u64 {
        self.sdl_timer_subsystem.performance_frequency()
    }

    /// Returns the number of milliseconds elapsed since SDL was initialized.
    pub fn millis(&self) -> u32 {
        self.sdl_timer_subsystem.ticks()
    }

    /// Delays (blocks) for about the number of milliseconds specified.
    pub fn delay(&mut self, millis: u32) {
        self.sdl_timer_subsystem.delay(millis);
    }
}
