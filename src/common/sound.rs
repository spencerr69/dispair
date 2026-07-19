use include_dir::{Dir, include_dir};
use rand::Rng;
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
                // let file = SOUNDS_DIR.get_file("005_ENEMY_KILL.wav").unwrap();
                // Cursor::new(file.contents())

                let mut rng = rand::rng();
                let choice = rng.random_range(0..10);
                match choice {
                    0..3 => {
                        let file = SOUNDS_DIR.get_file("010_KILL_1.wav").unwrap();
                        Cursor::new(file.contents())
                    }
                    3..4 => {
                        let file = SOUNDS_DIR.get_file("011_KILL_2.wav").unwrap();
                        Cursor::new(file.contents())
                    }
                    4..6 => {
                        let file = SOUNDS_DIR.get_file("012_KILL_3.wav").unwrap();
                        Cursor::new(file.contents())
                    }
                    6..9 => {
                        let file = SOUNDS_DIR.get_file("013_KILL_5.wav").unwrap();
                        Cursor::new(file.contents())
                    }
                    9..10 => {
                        let file = SOUNDS_DIR.get_file("014_KILL_7.wav").unwrap();
                        Cursor::new(file.contents())
                    }
                    _ => {
                        let file = SOUNDS_DIR.get_file("005_ENEMY_KILL.wav").unwrap();
                        Cursor::new(file.contents())
                    }
                }
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
