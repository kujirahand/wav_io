/// WAV file Header

/// Sample Rate - CD
pub const SAMPLE_RATE_CD: u32 = 44_100;
/// Sample Rate - DVD Audio
pub const SAMPLE_RATE_DVD_AUDIO: u32 = 48_000;
/// Sample Rate - AM Radio
pub const SAMPLE_RATE_AM_RADIO: u32 = 22_000;
/// Sample Rate - FM Radio
pub const SAMPLE_RATE_FM_AUDIO: u32 = 32_000;
/// Sample Rate - Tel
pub const SAMPLE_RATE_TEL: u32 = 8_000;

/// Sample Format
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum SampleFormat {
    Int,
    Float,
    WaveFromatALaw,
    WaveFormatMuLaw,
    SubFormat,
}

/// List Chunk Item
#[derive(Debug,Clone,PartialEq)]
pub struct ListChunkItem {
    pub id: String,
    pub value: String,
}
/// List Chunk Data
#[derive(Debug,Clone,PartialEq)]
pub struct ListChunk {
    pub items: Vec<ListChunkItem>,
}
impl ListChunk {
    pub fn make_block(&self) -> Vec<u8> {
        let mut block = Vec::new();
        for it in self.items.iter() {
            // chunk tag
            let chunk_tag_bytes = it.id.as_bytes();
            let mut chunk_tag: [u8; 4] = [32u8; 4];
            for (i, c) in chunk_tag_bytes.iter().enumerate() {
                if i >= 4 { break; }
                chunk_tag[i] = *c;
            }
            block.append(&mut chunk_tag.to_vec());
            // chunk size
            let mut flag_a = false;
            let mut chunk_size: u32 = it.value.len() as u32 + 1;
            if chunk_size % 2 != 0 {
                chunk_size += 1;
                flag_a = true;
            }
            block.append(&mut chunk_size.to_le_bytes().to_vec());
            // chunk value
            let bytes = it.value.as_bytes();
            // println!("chunk_size={}::bytes={}", chunk_size, bytes.len());
            block.append(&mut bytes.to_vec());
            block.push(0); // null
            if flag_a { block.push(0); }
        }
        block
    }
}

/// Wav file header
#[derive(Debug,Clone,PartialEq)]
pub struct WavHeader {
    pub sample_format: SampleFormat, // pcm=1
    pub channels: u16, // mono=1, stereo=2
    pub sample_rate: u32, // 44100Hz etc
    pub bits_per_sample: u16,
    pub list_chunk: Option<ListChunk>,
}

impl WavHeader {
    /// create WavHeader
    pub fn new() -> Self {
        Self::new_mono()
    }
    pub fn new_mono_i16_cd() -> Self {
        Self {
            sample_format: SampleFormat::Int,
            channels: 1,
            sample_rate: SAMPLE_RATE_CD,
            bits_per_sample: 16,
            list_chunk: None,
        }
    }
    pub fn new_mono_i16_radio() -> Self {
        Self {
            sample_format: SampleFormat::Int,
            channels: 1,
            sample_rate: SAMPLE_RATE_AM_RADIO,
            bits_per_sample: 16,
            list_chunk: None,
        }
    }
    pub fn new_mono_f32_cd() -> Self {
        Self {
            sample_format: SampleFormat::Float,
            channels: 1,
            sample_rate: SAMPLE_RATE_CD,
            bits_per_sample: 32,
            list_chunk: None,
        }
    }
    pub fn new_mono() -> Self {
        Self {
            sample_format: SampleFormat::Float,
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 32,
            list_chunk: None,
        }
    }
    pub fn new_stereo() -> Self {
        Self {
            sample_format: SampleFormat::Float,
            channels: 2,
            sample_rate: SAMPLE_RATE_CD,
            bits_per_sample: 32,
            list_chunk: None,
        }
    }
    pub fn set_int_format(&mut self) {
        self.sample_format = SampleFormat::Int;
    }
    pub fn set_float_format(&mut self) {
        self.sample_format = SampleFormat::Float;
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

