# wav_io for Rust

This crate provides utilities for reading and writing WAV files.

## Supported formats

- PCM 8-, 16-, 24-, and 32-bit integer
- PCM 32- and 64-bit float

## Functions

- [read](https://docs.rs/wav_io/latest/wav_io/reader/index.html)
- [write](https://docs.rs/wav_io/latest/wav_io/writer/index.html)
- [resample](https://docs.rs/wav_io/latest/wav_io/resample/index.html)
- [split by silence](https://docs.rs/wav_io/latest/wav_io/splitter/index.html)
- [generate sine waves](https://docs.rs/wav_io/latest/wav_io/tone/index.html)

## Installation

Add wav_io to your project:

```sh
cargo add wav_io
```

## Samples

```rust
use std::f32::consts::PI;
fn main() {
    // make sine wave
    let head = wav_io::new_mono_header();
    let mut samples: Vec<f32> = vec![];
    for t in 0..head.sample_rate {
        let v = ((t as f32 / head.sample_rate as f32) * 440.0 * 2.0 * PI).sin() * 0.6;
        samples.push(v);
    }
    // write to file
    let mut file_out = std::fs::File::create("./sine.wav").unwrap();
    wav_io::write_to_file(&mut file_out, &head, &samples).unwrap();
}
```

## Examples

- [Samples](https://docs.rs/wav_io/latest/wav_io/index.html)

## Links

- [Docs](https://docs.rs/wav_io/latest/wav_io/)
- [Repository](https://github.com/kujirahand/wav_io)
