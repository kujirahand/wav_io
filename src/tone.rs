/// tone generator

use std::f32::consts::PI;

/// Tone note
#[derive(Debug)]
pub struct Note {
    pub no: i32,
    pub len: i32,
    pub vel: f32, // 0.0 - 1.0
}

/// Tone options
pub struct ToneOptions {
    pub sample_rate: u32,
    pub bpm: f32,
    pub volume: f32, // 0.0 - 1.0
}
impl ToneOptions {
    pub fn new() -> Self {
        ToneOptions {
            sample_rate: 44000,
            bpm: 120.0,
            volume: 0.9,
        }
    }
}

/// write notes
pub fn write_notes(samples: &mut Vec<f32>, notes: Vec<Note>, opt: &ToneOptions) {
    // A short fade-in/out prevents clicks caused by abrupt waveform jumps.
    let env_ms = 5.0_f32;
    let env_samples = ((opt.sample_rate as f32 * env_ms) / 1000.0) as usize;

    for note in notes.iter() {
        let len = (4.0 / note.len as f32 * (60.0 / opt.bpm) * opt.sample_rate as f32) as usize;
        let tone = if note.no < 0 { 0.0 } else {
            440.0 * 2.0f32.powf((note.no - 69) as f32 / 12.0)
        };
        let env_len = env_samples.min(len.saturating_div(2));
        for t in 0..len {
            let a = t as f32 / opt.sample_rate as f32;
            let mut env = 1.0_f32;
            if env_len > 0 {
                if t < env_len {
                    env = t as f32 / env_len as f32;
                } else if t >= len - env_len {
                    env = (len - t - 1) as f32 / env_len as f32;
                }
            }
            let v = (a * tone * 2.0 * PI).sin() * opt.volume * note.vel * env;
            samples.push(v);
        }
    }
}

/// write mml
pub fn write_mml(samples: &mut Vec<f32>, mml: &str, opt: &ToneOptions) {
    let notes = parse_mml(String::from(mml));
    write_notes(samples, notes, opt);
}

/// parse mml string
fn parse_mml(src: String) -> Vec<Note> {
    let mut result = vec![];
    let mut octave = 5;
    let mut length = 4;
    let mut vel = 8; // 0-9
    let mut it = src.chars();
    while let Some(ch) = it.next() {
        match ch {
            'a'..='g' => { // Note
                let note = match ch {
                    'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5, 
                    'g' => 7, 'a' => 9, 'b' => 11, _ => 0,
                };
                let no = note + octave * 12;
                result.push(Note{no, len: length, vel: vel as f32 / 9.0});
            },
            'r' => result.push(Note{no: -1, len: length, vel: 0.0}), // Rest
            'o' => { // Octave
                let v = it.next().expect("should be number");
                let o = v as i32 - '0' as i32;
                if o >= 0 && o < 9 { octave = o; }
            },
            'l' => { // Length
                let v = it.next().expect("should be number");
                let l = v as i32 - '0' as i32;
                if l >= 1 && l <= 9 { length = l; }
            },
            'v' => { // Velocity
                let n = it.next().expect("should be number");
                let v = n as i32 - '0' as i32;
                if v >= 0 && v <= 9 { vel = v; }
            },
            _ => {}, // skip
        };
    }
    result
}
