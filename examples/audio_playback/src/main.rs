use std::path::Path;

use anyhow::Result;
use sdl2::keyboard::Scancode;

use libretrogd::audio::*;
use libretrogd::graphics::*;
use libretrogd::system::*;

#[derive(Debug, Copy, Clone)]
struct AudioChannelStatus {
    size: usize,
    position: usize,
    playing: bool
}

fn load_and_convert_wav(path: &Path) -> Result<AudioBuffer> {
    let sound = AudioBuffer::load_wav_file(path)?;
    let original_spec = *sound.spec();
    let sound = sound.convert(&TARGET_AUDIO_SPEC)?;
    let final_spec = *sound.spec();
    if original_spec != final_spec {
        println!("{:?} was converted from {:?} to {:?}", path, original_spec, final_spec);
    } else {
        println!("{:?} did not need to be converted from {:?}", path, original_spec);
    }
    Ok(sound)
}

fn main() -> Result<()> {
    let mut system = SystemBuilder::new().window_title("Audio Playback").vsync(true).build()?;

    let mut is_running = true;

    let sound1 = load_and_convert_wav(Path::new("./assets/pickup-coin.wav"))?;
    let sound2 = load_and_convert_wav(Path::new("./assets/powerup.wav"))?;
    let sound3 = load_and_convert_wav(Path::new("./assets/explosion.wav"))?;
    let sound4 = load_and_convert_wav(Path::new("./assets/jump.wav"))?;
    let sound5 = load_and_convert_wav(Path::new("./assets/laser-shoot.wav"))?;

    let mut statuses = [AudioChannelStatus { size: 0, position: 0, playing: false }; NUM_CHANNELS];

    while is_running {
        system.do_events_with(|event| {
            if let sdl2::event::Event::Quit { .. } = event {
                is_running = false;
            }
        });

        let mut audio_device = system.audio.lock();

        if system.keyboard.is_key_pressed(Scancode::Escape) {
            is_running = false;
        }

        if system.keyboard.is_key_pressed(Scancode::Num1) {
            audio_device.play_buffer(&sound1, false)?;
        }

        if system.keyboard.is_key_pressed(Scancode::Num2) {
            audio_device.play_buffer(&sound2, false)?;
        }

        if system.keyboard.is_key_pressed(Scancode::Num3) {
            audio_device.play_buffer(&sound3, false)?;
        }

        if system.keyboard.is_key_pressed(Scancode::Num4) {
            audio_device.play_buffer(&sound4, false)?;
        }

        if system.keyboard.is_key_pressed(Scancode::Num5) {
            audio_device.play_buffer(&sound5, false)?;
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

        system.video.print_string("Audio Channels", 16, 16, FontRenderOpts::Color(14), &system.font);

        let mut y = 32;
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
                &system.font
            );
            y += 16;
        }

        system.display()?;
    }

    Ok(())
}
