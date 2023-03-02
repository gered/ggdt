use std::path::Path;

use anyhow::Result;

use ggdt::audio::*;
use ggdt::graphics::*;
use ggdt::system::*;
use ggdt::utils::rnd_value;

#[derive(Debug, Copy, Clone)]
struct AudioChannelStatus {
	size: usize,
	position: usize,
	playing: bool,
}

fn load_and_convert_wav(path: &Path, target_spec: &AudioSpec) -> Result<AudioBuffer> {
	let sound = AudioBuffer::load_wav_file(path)?;
	let original_spec = *sound.spec();
	let sound = sound.convert(target_spec)?;
	let final_spec = *sound.spec();
	if original_spec != final_spec {
		println!("{:?} was converted from {:?} to {:?}", path, original_spec, final_spec);
	} else {
		println!("{:?} did not need to be converted from {:?}", path, original_spec);
	}
	Ok(sound)
}

pub struct SineWaveGenerator {
	t: usize,
}

impl SineWaveGenerator {
	pub fn new() -> Self {
		SineWaveGenerator {
			t: 0
		}
	}
}

impl AudioGenerator for SineWaveGenerator {
	fn gen_sample(&mut self, position: usize) -> Option<u8> {
		const MAX_TIME: usize = AUDIO_FREQUENCY_22KHZ as usize * 3;  // 3 seconds
		if self.t < MAX_TIME {
			let sample = (self.t as f64 * 0.25).sin() * 80.0;
			self.t += 1;
			Some((sample + 128.0) as u8)
		} else {
			None
		}
	}
}

fn main() -> Result<()> {
	let mut system = SystemBuilder::new().window_title("Audio Playback").vsync(true).build()?;

	let mut using_queue_commands = false;
	let mut volume = 1.0;

	let sounds = [
		load_and_convert_wav(Path::new("./assets/pickup-coin.wav"), system.audio.spec())?,
		load_and_convert_wav(Path::new("./assets/powerup.wav"), system.audio.spec())?,
		load_and_convert_wav(Path::new("./assets/explosion.wav"), system.audio.spec())?,
		load_and_convert_wav(Path::new("./assets/jump.wav"), system.audio.spec())?,
		load_and_convert_wav(Path::new("./assets/laser-shoot.wav"), system.audio.spec())?,
	];

	let mut statuses = [AudioChannelStatus { size: 0, position: 0, playing: false }; NUM_CHANNELS];

	while !system.do_events() {
		if system.input_devices.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		let mut audio_device = system.audio.lock();
		audio_device.volume = volume;

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num1) {
			if using_queue_commands {
				system.audio_queue.play_buffer(&sounds[0], false);
			} else {
				audio_device.play_buffer(&sounds[0], false)?;
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num2) {
			if using_queue_commands {
				system.audio_queue.play_buffer(&sounds[1], false);
			} else {
				audio_device.play_buffer(&sounds[1], false)?;
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num3) {
			if using_queue_commands {
				system.audio_queue.play_buffer(&sounds[2], false);
			} else {
				audio_device.play_buffer(&sounds[2], false)?;
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num4) {
			if using_queue_commands {
				system.audio_queue.play_buffer(&sounds[3], false);
			} else {
				audio_device.play_buffer(&sounds[3], false)?;
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num5) {
			if using_queue_commands {
				system.audio_queue.play_buffer(&sounds[4], false);
			} else {
				audio_device.play_buffer(&sounds[4], false)?;
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num6) {
			if using_queue_commands {
				system.audio_queue.play_generator(Box::new(SineWaveGenerator::new()), false);
			} else {
				audio_device.play_generator(Box::new(SineWaveGenerator::new()), false);
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::Num7) {
			let index = rnd_value(0, sounds.len() - 1);
			if using_queue_commands {
				system.audio_queue.play_buffer_on_channel(7, &sounds[index], false)?;
			} else {
				audio_device.play_buffer_on_channel(7, &sounds[index], false)?;
			}
		}

		if system.input_devices.keyboard.is_key_pressed(Scancode::S) {
			if using_queue_commands {
				system.audio_queue.stop_all();
			} else {
				audio_device.stop_all();
			}
		}

		system.audio_queue.apply_to_device(&mut audio_device)?;

		if system.input_devices.keyboard.is_key_pressed(Scancode::KpMinus) {
			volume -= 0.1;
		}
		if system.input_devices.keyboard.is_key_pressed(Scancode::KpPlus) {
			volume += 0.1;
		}
		if system.input_devices.keyboard.is_key_pressed(Scancode::Q) {
			using_queue_commands = !using_queue_commands;
		}

		for index in 0..NUM_CHANNELS {
			let channel = &audio_device[index];
			let mut status = &mut statuses[index];
			status.playing = channel.playing;
			status.position = channel.position;
			status.size = channel.data.len();
		}

		drop(audio_device);

		system.video.clear(0);

		system.video.print_string(&format!("Volume: {:2.2}", volume), 16, 16, FontRenderOpts::Color(10), &system.font);
		system.video.print_string(
			if using_queue_commands {
				"Queueing Commands"
			} else {
				"Direct Commands"
			},
			160, 16, FontRenderOpts::Color(9), &system.font,
		);

		system.video.print_string("Audio Channels", 16, 32, FontRenderOpts::Color(14), &system.font);

		let mut y = 48;
		for index in 0..NUM_CHANNELS {
			let status = &statuses[index];
			system.video.print_string(
				&format!(
					"channel {} - {} {}",
					index,
					if status.playing { "playing" } else { "not playing" },
					if status.playing { String::from(format!("{} / {}", status.position, status.size)) } else { String::new() }
				),
				16, y,
				FontRenderOpts::Color(15),
				&system.font,
			);
			y += 16;
		}

		system.display()?;
	}

	Ok(())
}
