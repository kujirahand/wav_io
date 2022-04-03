/// wav file header

/*
const SAMPLE_RATE_CD: u32 = 44_100;
const SAMPLE_RATE_DVD_AUDIO: u32 = 48_000;
const SAMPLE_RATE_AM_RADIO: u32 = 22_000;
const SAMPLE_RATE_FM_AUDIO: u32 = 32_000;
const SAMPLE_RATE_TEL: u32 = 8_000;
*/

/// Sample Format
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum SampleFormat {
    Int,
    Float,
    WaveFromatALaw,
    WaveFormatMuLaw,
    SubFormat,
}

/// Wav file header
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct WavHeader {
    pub sample_format: SampleFormat, // pcm=1
    pub channels: u16, // mono=1, stereo=2
    pub sample_rate: u32, // 44100Hz etc
    pub bits_per_sample: u16,
}

impl WavHeader {
    /// create WavHeader
    pub fn new() -> Self {
        Self::new_mono()
    }
    pub fn new_mono() -> Self {
        Self {
            sample_format: SampleFormat::Float,
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 32,
        }
    }
    pub fn new_stereo() -> Self {
        Self {
            sample_format: SampleFormat::Float,
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 32,
        }
    }
}

/// Wav Data
#[derive(Debug,Clone,PartialEq)]
pub struct WavData {
    pub header: WavHeader,
    pub samples: Vec<f32>,
}

impl WavData {
    pub fn new(header: WavHeader, samples: Vec<f32>) -> Self {
        Self {header, samples}
    }
}
