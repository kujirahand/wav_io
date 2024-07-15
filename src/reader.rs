/// Wav file reader

use std::fs::File;
use std::io::{Cursor, Read};
use crate::header::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid chunk tag, expected '{expected:?}', found '{found:?}'")]
    InvalidTag {
        expected: &'static str,
        found: String,
    },
    #[error("Invalid chunk attribute, attribute {attribute:?} must be greater than {expected:?}, found {found:?} instead")]
    InvalidChunkAttribute {
        attribute: &'static str,
        expected: u32,
        found: u32,
    },
    #[error("Invalid chunk attribute, attribute {attribute} must be on of {expected:?}, found 0x{found:02x}")]
    InvalidChunkAttributeRange {
        attribute: &'static str,
        expected: &'static [u32],
        found: u32,
    },
    #[error("Unsupported wav-format, attribute {attribute} must be one of {expected:?}, found 0x{found:02x}")]
    UnsupportedWav {
        attribute: &'static str,
        expected: &'static [u32],
        found: u32,
    },
    #[error("Unsupported wav encoding, module only supports PCM data")]
    UnsupportedEncoding,
    #[error("Failed to open file")]
    FileOpen {
        #[source]
        source: std::io::Error,
    },
    #[error("Unsupported system, please use a 32-bit system or higher")]
    UnsupportedSystem,
    #[error("Unable to read data")]
    ReadFail {
        #[source]
        source: std::io::Error,
    },
}

/// Get header and samples from file
pub fn from_file(file: File) -> Result<WavData, DecodeError> {
    // read file
    let mut r: Reader = match Reader::from_file(file) {
        Err(err) => return Err(err),
        Ok(r) => r,
    };
    // read header
    let header = match r.read_header() {
        Err(err) => return Err(err),
        Ok(head) => head,
    };
    // read samples
    let samples = match r.get_samples_f32() {
        Err(err) => return Err(err),
        Ok(samples) => samples,
    };
    Ok(WavData{header, samples})
}

/// Get header and samples from file path
pub fn from_file_str(file_path: &str) -> Result<WavData, DecodeError> {
    let f = match File::open(file_path) {
        Ok(f) => f,
        Err(err) => return Err(DecodeError::FileOpen { source: err }),
    };
    from_file(f)
}

/// Wav file reader for binary
pub struct Reader {
    pub cur: Cursor<Vec<u8>>,
    pub header: Option<WavHeader>,
}
impl Reader {    
    /// Create Reader Object from wav file
    pub fn from_file(file: File) -> Result<Reader, DecodeError> {
        let mut data: Vec<u8> = Vec::new();
        let mut f = file;
        match f.read_to_end(&mut data) {
            Ok(_) => {},
            Err(err) => return Err(DecodeError::ReadFail { source: err }),
        };
        Self::from_vec(data)
    }
    /// Crate Reader Object from Vec
    pub fn from_vec(data: Vec<u8>) -> Result<Reader, DecodeError> {
        let reader = Reader {
            cur: Cursor::new(data),
            header: None,
        };
        Ok(reader)
    }
    /// Read Wav file header
    pub fn read_header(&mut self) -> Result<WavHeader, DecodeError> {
        let mut header = WavHeader::new();
        // RIFF header
        let riff_tag = self.read_str4();
        if riff_tag != "RIFF" {
            return Err(DecodeError::InvalidTag { expected: "RIFF", found: riff_tag.to_string() });
        }
        // size
        let chunk_size = self.read_u32().unwrap_or(0);
        if chunk_size < 8 {
            return Err(DecodeError::InvalidChunkAttribute {
                attribute: "chunk size",
                expected: 7,
                found: chunk_size,
            });
        }
        // should be WAVE
        let wave_tag = self.read_str4();
        if  wave_tag != "WAVE" {
            return Err(DecodeError::InvalidTag { expected: "WAVE", found: riff_tag.to_string() });
        }

        // check for a possible LIST chunk
        // if there is one, skip it
        let _ = self.read_list_chunk(&mut header);

        // fmt
        let fmt_tag = self.read_str4();
        if fmt_tag != "fmt " {
            return Err(DecodeError::InvalidTag { expected: "fmt ", found: riff_tag.to_string() });
        }
        let chunk_size = self.read_u32().unwrap_or(0);
        // audio format
        let format_tag = self.read_u16().unwrap_or(0);
        match format_tag {
            0x0001 => header.sample_format = SampleFormat::Int,
            0x0003 => header.sample_format = SampleFormat::Float,
            0x0006 => header.sample_format = SampleFormat::WaveFromatALaw,
            0x0007 => header.sample_format = SampleFormat::WaveFormatMuLaw,
            0xFFFE => header.sample_format = SampleFormat::SubFormat,
            0x0055 => return Err(DecodeError::UnsupportedWav {
                attribute: "format tag (0x0055: MP3)",
                expected: &[0x0001, 0x0003, 0x0006, 0x0007, 0xFFFE],
                found: format_tag as u32,
            }),
            _ => return Err(DecodeError::InvalidChunkAttributeRange {
                attribute: "format tag",
                expected: &[0x0001, 0x0003, 0x0006, 0x0007, 0xFFFE],
                found: format_tag as u32,
            }),
        }
        // channels
        let ch = self.read_u16().unwrap_or(0);
        if 1 <= ch && ch <= 2 {
            header.channels = ch;
        }
        // sample_rate
        header.sample_rate = self.read_u32().unwrap_or(0);
        if header.sample_rate < 32 {
            return Err(DecodeError::InvalidChunkAttribute {
                attribute: "sample rate",
                expected: 31,
                found: header.sample_rate,
            });
        }
        // ave bytes per sec (sample_rate * bits * channels)
        let bytes_per_sec = self.read_u32().unwrap_or(0);
        if bytes_per_sec < 8 {
            return Err(DecodeError::InvalidChunkAttribute {
                attribute: "bytes per second",
                expected: 7,
                found: header.sample_rate,
            });
        }
        // nBlockAlign (channels * bits  / 8)
        let _data_block_size = self.read_u16().unwrap_or(0);
        // Bits per sample
        let bits_per_sample = self.read_u16().unwrap_or(0);
        if bits_per_sample < 8 {
            return Err(DecodeError::InvalidChunkAttribute {
                attribute: "bits per sample",
                expected: 7,
                found: header.sample_rate,
            });
        }
        header.bits_per_sample = bits_per_sample;
        // println!("chunk_size={}",chunk_size);
        let pos = self.cur.position() + chunk_size as u64 - 16;
        self.cur.set_position(pos);

        // check for a possible LIST chunk
        // if there is one, skip it
        let _ = self.read_list_chunk(&mut header);

        // set to header
        self.header = Some(header.clone());
        Ok(header)
    }

    /// Read a LIST chunk
    /// This function will only progres the internal data cursor when `Ok()` is returned
    /// In the case of `Err()`, the cursor will not have moved
    pub fn read_list_chunk(&mut self, header: &mut WavHeader) -> Result<usize, DecodeError> {
        // keep track of the position, in case we error, we can jump back
        let begin_position = self.cur.position();

        // check the tag
        let info_tag = self.read_str4();
        if info_tag != "LIST" {
            self.cur.set_position(begin_position);
            return Err(DecodeError::InvalidTag {
                expected: "LIST",
                found: info_tag.to_string()
            });
        }
        // retrieve the info size and convert to an usize
        let Some(read_size) = self.read_u32() else {
            self.cur.set_position(begin_position);
            return Err(DecodeError::ReadFail { source: std::io::Error::new(std::io::ErrorKind::Other, "Unable to read u32") })
        };
        let Ok(read_size) = read_size.try_into() else {
            self.cur.set_position(begin_position);
            return Err(DecodeError::UnsupportedSystem)
        };

        // read the data and return it
        let mut data = vec![0; read_size];
        match self.cur.read_exact(&mut data) {
            Ok(_) => {
                Ok(self.analize_list_chunk(data, header))
            },
            Err(err) => {
                self.cur.set_position(begin_position);
                Err(DecodeError::ReadFail { source: err })
            }
        }
    }

    pub fn analize_list_chunk(&mut self, data: Vec<u8>, header: &mut WavHeader) -> usize {
        let data_len = data.len() as u64;
        let mut cur = Cursor::new(data);
        // read tag
        let mut chunk_tag = [0u8; 4];
        let chunk_tag = match cur.read_exact(&mut chunk_tag) {
            Ok(_) => String::from_utf8_lossy(&chunk_tag),
            Err(_) => return 0, // ERROR LIST CHUNK
        };
        if chunk_tag != "INFO" {
            return 0;
        }
        /*
        // (ref) https://www.recordingblogs.com/wiki/list-chunk-of-a-wave-file
        'IART' artist name
        'INAM' name of the song
        'IPRD' product name (album name)
        'IGNR' genre
        'ICMT' comment
        'ITRK' track number
        'ICRD' creation date
        'ISFT' software
        'ITOC' CDのTOC(Table of Contents)
        */
        let mut items = vec![];
        let mut result = 0;
        while cur.position() < data_len {
            // read chunk tag
            let mut chunk_tag = [0u8; 4];
            let chunk_tag = match cur.read_exact(&mut chunk_tag) {
                Ok(_) => String::from_utf8_lossy(&chunk_tag),
                Err(_) => break,
            };
            // read info len
            let mut chunk_size = [0u8; 4];
            let chunk_size = match cur.read_exact(&mut chunk_size) {
                Ok(_) => u32::from_le_bytes(chunk_size),
                Err(_) => break,
            };
            let mut data = vec![0; chunk_size as usize];
            let data = match cur.read_exact(&mut data) {
                Ok(_) => String::from_utf8_lossy(&data),
                Err(_) => break,
            };
            // println!("chunk_tag={:?}::{}::{}", chunk_tag, chunk_size, data);
            let item = ListChunkItem {
                id: chunk_tag.trim_end_matches('\0').to_string(),
                value: data.trim_end_matches('\0').to_string(),
            };
            items.push(item);
            result += 1;
        }
        header.list_chunk = Some(ListChunk{items});
        result
    }

    pub fn get_samples_f32(&mut self) -> Result<Vec<f32>, DecodeError> {
        let mut result:Vec<f32> = Vec::new();
        loop {
            // read chunks
            let chunk_tag = self.read_str4();
            if chunk_tag == "" { break; }
            let size = self.read_u32().unwrap_or(0) as u64;
            // todo: check tag
            // println!("[info] tag={:?}::{}", chunk_tag, size);
            if size == 0 { continue }
            // data?
            if chunk_tag != "data" {
                self.cur.set_position(self.cur.position() + size);
                continue;
            }
            // read wav data
            let h = &self.header.clone().unwrap();
            let num_sample = (size / (h.bits_per_sample / 8) as u64) as u64;
            match h.sample_format {
                // float
                SampleFormat::Float => {
                    match h.bits_per_sample {
                        32 => {
                            for _ in 0..num_sample {
                                let lv = self.read_f32().unwrap_or(0.0);
                                result.push(lv);
                            }
                        },
                        64 => {
                            for _ in 0..num_sample {
                                let lv = self.read_f64().unwrap_or(0.0);
                                result.push(lv as f32); // down to f32
                            }
                        },
                        _ => return Err(DecodeError::UnsupportedWav {
                            attribute: "bits per float sample",
                            expected: &[32, 64],
                            found: h.bits_per_sample as u32,
                        }),
                    }
                },
                // int
                SampleFormat::Int => {
                    match h.bits_per_sample {
                        8 => {
                            for _ in 0..num_sample {
                                // 0..255
                                let lv = self.read_u8().unwrap_or(0);
                                let fv = (lv - 128) as f32;
                                result.push(fv);
                            }
                        },
                        16 => {
                            for _ in 0..num_sample {
                                let lv = self.read_i16().unwrap_or(0);
                                let fv = lv as f32 / (0xFFFF as f32 / 2.0);
                                result.push(fv);
                            }
                        },
                        24 => {
                            for _ in 0..num_sample {
                                let lv = self.read_i24().unwrap_or(0);
                                let fv = lv as f32 / (0xFFFFFF as f32 / 2.0);
                                result.push(fv);
                            }
                        },
                        32 => {
                            for _ in 0..num_sample {
                                let lv = self.read_i32().unwrap_or(0);
                                let fv = lv as f32 / (0xFFFFFFFFu32 as f32 / 2.0);
                                result.push(fv);
                            }
                        },
                        _ => return Err(DecodeError::UnsupportedWav {
                            attribute: "bits per integer sample",
                            expected: &[8, 16, 24, 32],
                            found: h.bits_per_sample as u32,
                        }),
                    }
                },
                _ => return Err(DecodeError::UnsupportedEncoding),
            }
        }
        Ok(result)
    }

    pub fn read_str4(&mut self) -> String {
        let mut buf = [0u8; 4];
        match self.cur.read(&mut buf) {
            Ok(sz) => {
                if sz < 4 {
                    return String::from("");
                }
            },
            Err(_) => return String::from(""),
        }
        let s = String::from_utf8_lossy(&buf);
        String::from(s)
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        match self.read_u32() {
            Some(v) => Some(f32::from_bits(v)),
            None => None,
        }
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        match self.read_u64() {
            Some(v) => Some(f64::from_bits(v)),
            None => None,
        }
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        let mut buf = [0u8; 8];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(u64::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        let mut buf = [0u8; 4];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(u32::from_le_bytes(buf))
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        let mut buf = [0u8; 4];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(i32::from_le_bytes(buf))
    }

    pub fn read_u24(&mut self) -> Option<u32> {
        let mut buf = [0u8; 3];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let result = 
            (buf[0] as u32) << 0 | 
            (buf[1] as u32) << 8 |
            (buf[2] as u32) << 16;
        Some(result)
    }

    pub fn read_i24(&mut self) -> Option<i32> {
        let mut buf = [0u8; 3];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let buf4 = [0, buf[0], buf[1], buf[2]];
        Some(i32::from_le_bytes(buf4) >> 8)
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        let mut buf = [0u8; 2];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let result = 
            (buf[0] as u16) << 0 |
            (buf[1] as u16) << 8;
        Some(result)
    }

    pub fn read_i16(&mut self) -> Option<i16> {
        let mut buf = [0u8; 2];
        match self.cur.read(&mut buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(i16::from_le_bytes(buf))
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        let mut buf = [0u8; 1];
        match self.cur.read(&mut buf) {
            Ok(_) => Some(buf[0]),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    
    fn read_it() {
        let mut r = Reader::from_vec(vec![1,0]).unwrap();
        assert_eq!(Some(1), r.read_i16());
        let mut r = Reader::from_vec(vec![0,1]).unwrap();
        assert_eq!(Some(0x100), r.read_i16());

        let mut r = Reader::from_vec(vec![0xFF,0xFF]).unwrap();
        assert_eq!(Some(-1), r.read_i16());
        let mut r = Reader::from_vec(vec![0xFE,0xFF]).unwrap();
        assert_eq!(Some(-2), r.read_i16());
        let mut r = Reader::from_vec(vec![0xFD,0xFF]).unwrap();
        assert_eq!(Some(-3), r.read_i16());

        let mut r = Reader::from_vec(vec![0xFF,0xFF, 0xFF]).unwrap();
        assert_eq!(Some(-1), r.read_i24());
    }
}

