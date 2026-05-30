use include_dir::{Dir, include_dir};
use rodio::source::Amplify;
use rodio::{Decoder, MixerDeviceSink, Source};
use std::io::Cursor;

static SOUNDS_DIR: Dir<'static> = include_dir!("assets/sounds");
pub enum SoundEffect {
    Hit,
    Flash,
    RoundStart,
    Ignite,
    EnemyKill,
}

impl SoundEffect {
    pub fn get_file(&self) -> Cursor<&'static [u8]> {
        match self {
            SoundEffect::Hit => {
                let file = SOUNDS_DIR.get_file("001_HIT.wav").unwrap();

                Cursor::new(file.contents())
            }

            SoundEffect::Flash => {
                let file = SOUNDS_DIR.get_file("002_FLASH.wav").unwrap();
                Cursor::new(file.contents())
            }
            SoundEffect::RoundStart => {
                let file = SOUNDS_DIR.get_file("003_ROUND_START.wav").unwrap();
                Cursor::new(file.contents())
            }
            SoundEffect::Ignite => {
                let file = SOUNDS_DIR.get_file("004_IGNITE.wav").unwrap();
                Cursor::new(file.contents())
            }
            SoundEffect::EnemyKill => {
                let file = SOUNDS_DIR.get_file("005_ENEMY_KILL.wav").unwrap();
                Cursor::new(file.contents())
            }
        }
    }

    pub fn decoded(&self) -> Amplify<Decoder<Cursor<&'static [u8]>>> {
        Decoder::try_from(self.get_file())
            .expect("Couldn't decode file.")
            .amplify(0.1)
    }
}

pub struct SoundWrangler {
    device_sink: MixerDeviceSink,
}

impl Default for SoundWrangler {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundWrangler {
    pub fn new() -> Self {
        let sink = rodio::DeviceSinkBuilder::open_default_sink().expect("Error opening sound");

        Self { device_sink: sink }
    }

    pub fn play(&self, effect: SoundEffect) {
        let source = effect.decoded();

        self.device_sink.mixer().add(source);
    }
}
