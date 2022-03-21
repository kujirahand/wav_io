pub mod header;
pub mod reader;
pub mod writer;
pub mod utils;
pub mod splitter;

use std::fs::File;

fn main() {
    // read
    let file_in = File::open("./i32.wav").unwrap();
    let mut wav = reader::read_from_file(file_in).unwrap();
    println!("header={:?}", wav.header);
    println!("samples.len={}", wav.samples.len());
    // write
    let mut file_out = File::create("./out.wav").unwrap();
    writer::write(&mut file_out, &mut wav).unwrap();   
}

/*
fn main() {
    iotest();
    // split
    let file_in = File::open("./i32.wav").unwrap();
    let mut wav = reader::read_from_file(file_in).unwrap();
    
    println!("===split=== {:?}", wav.header);
    let sub_vec = splitter::split_samples(&mut wav.samples, wav.header.sample_rate, true);
    for (i, range) in sub_vec.iter().enumerate() {
        let mut w = Writer::new();
        let parts = splitter::sub_samples(&wav.samples, *range);
        w.from_scratch(&wav.header, &parts).unwrap();
        let fname = format!("./sub-{}.wav", i);
        println!("split.saved={}", fname);
        let mut f = File::create(fname).unwrap();
        w.to_file(&mut f).unwrap();
    }
} 

fn iotest() {
    // file_in / file_out
    let file_in = File::open("./i32.wav").unwrap();
    let mut file_out = File::create("./test-out.wav").unwrap();
    
    // read
    println!("hello");
    let mut r =Reader::from_file(file_in).unwrap();
    r.read_header().unwrap();
    println!("{:?}", r.header);
    let samples = r.get_samples_f32().unwrap();
    println!("samples={}", samples.len());

    // write
    let mut w = Writer::new();
    let mut head = r.header.unwrap();
    let samples_mono = utils::stereo_to_mono(samples);

    head.channels = 1;
    head.bits_per_sample = 32;
    head.sample_format = SampleFormat::Int;
    let mut samples2 = utils::resample(samples_mono, head.sample_rate, 44_800);
    head.sample_rate = 32_000;
    w.from_scratch(&head, &samples2).unwrap();
    w.to_file(&mut file_out).unwrap();
}

*/