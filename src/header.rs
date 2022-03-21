// wav file header

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum SampleFormat {
    Int,
    Float,
    WaveFromatALaw,
    WaveFormatMuLaw,
    SubFormat,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub struct WavHeader {
    pub sample_format: SampleFormat, // pcm=1
    pub channels: u16, // mono=1, stereo=2
    pub sample_rate: u32, // 44100Hz etc
    pub bits_per_sample: u16,
}

impl WavHeader {
    pub fn new() -> Self {
        Self {
            sample_format: SampleFormat::Int,
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 32,
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct WavData {
    pub header: WavHeader,
    pub samples: Vec<f32>,
}
