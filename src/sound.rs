use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use rodio::source::{Source, Buffered};


#[allow(dead_code)]
pub enum Sound {
    Click = 0,
    FireballLaunch = 1,
    FireballHit = 2,
}


pub struct SoundPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sinks: Vec <Sink>,
    sources: Vec <Buffered<Decoder<BufReader<File>>>>
}


impl SoundPlayer {

    pub fn new() -> SoundPlayer {

        // stream must live as long as the sink
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        // Play the sound directly on the device
        // stream_handle.play_raw(source.clone().convert_samples());

        let mut sources = Vec::new();

        sources.push(load_sound("resources/sfx/click.wav"));
        sources.push(load_sound("resources/sfx/fireball_launch.wav"));
        sources.push(load_sound("resources/sfx/fireball_hit_3a.wav"));

        let mut sinks = Vec::new();

        // we need as many sinks as there should be sounds played in parallel

        for _i in 0..4 {
            sinks.push(Sink::try_new(&stream_handle).unwrap());
        }

        SoundPlayer {
            _stream: stream,
            _stream_handle: stream_handle,
            sinks,
            sources,
        }
    }

    pub fn play(&self, id: Sound, volume: f32) {
        let index = id as usize;
        self.play_sound(index, volume)
    }

    pub fn play_sound(&self, index: usize, volume: f32) {

        println!("Playing sound {}", index);

        for sink in &self.sinks {
            if sink.empty() {
                sink.set_volume(volume);
                sink.append(self.sources[index].clone());
                return;
            }
        }
    }
}


fn load_sound(path: &str) -> Buffered<Decoder<BufReader<File>>> {
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open(path).unwrap());

    // Decode that sound file into a source
    Decoder::new(file).unwrap().buffered()
}