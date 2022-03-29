# wav_io 

This is a crate for reading and writing wav file.

## Suported format:

- PCM 8, 16, 24, 32 bits Int
- PCM 32, 64 bits Float

## Functions

- read / write
- resamle
- split by silence
- make sine wave

## Example

```
fn read_write() {
    // read
    let file_in = File::open("./tone.wav").unwrap();
    let wav = reader::from_file(file_in).unwrap();
    println!("header={:?}", wav.header);
    println!("samples.len={}", wav.samples.len());
    
    // write
    let mut file_out = File::create("./test-out.wav").unwrap();
    writer::to_file(&mut file_out, &wav).unwrap(); 
}

fn write_tone() {
    // write tone
    let header = WavHeader::new_mono();
    let mut samples = vec![];
    for t in 0..header.sample_rate {
        let v = ((t as f32 / header.sample_rate as f32) * 440.0 * 2.0 * PI).sin() * 0.6;
        samples.push(v);
    }
    let mut file_out = File::create("./tone.wav").unwrap();
    writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();

    // melody
    let mut header = WavHeader::new_mono();
    let mut samples = vec![];
    let opt = tone::ToneOptions::new();
    header.sample_rate = opt.sample_rate;
    tone::write_mml(&mut samples, "l4 cege cege cege c1", &opt);
    let mut file_out = File::create("./melody.wav").unwrap();
    writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();
}

fn resample() {    
    // resample
    let file_in = File::open("./tone.wav").unwrap();
    let wav = reader::from_file(file_in).unwrap();
    let new_sample_rate = 16_000;
    let mut file_out = File::create("./tone-resample.wav").unwrap();
    let samples2 = resample::linear(wav.samples, wav.header.channels, wav.header.sample_rate, new_sample_rate);
    let mut wav2 = header::WavData{header: wav.header, samples: samples2};
    wav2.header.sample_rate = new_sample_rate;
    writer::to_file(&mut file_out, &wav2).unwrap();
    println!("resample.writer={:?}", wav2.header);
}

fn split() {
    // split
    let file_in = File::open("./test.wav").unwrap();
    let mut wav = reader::from_file(file_in).unwrap();
    let mut samples = wav.samples;
    if wav.header.channels >= 2 {
        samples = utils::stereo_to_mono(samples);
        wav.header.channels = 1;
    }
    let range_vec = splitter::split_samples(&mut samples, wav.header.sample_rate, true);
    for (i, range) in range_vec.iter().enumerate() {
        let fname = format!("sub-{}.wav", i);
        println!("split_samples={}", fname);
        let mut file_out = File::create(fname).unwrap();
        let sub = splitter::sub_samples(&samples, *range);
        let wav = header::WavData{header: wav.header, samples: sub};
        writer::to_file(&mut file_out, &wav).unwrap();
    }
}
```
