/// Wav file reader

use std::fs::File;
use std::io::{Cursor, Read};
use crate::header::*;

const ERR_INVALID_FORMAT: &str = "invalid wav format";
const ERR_UNSUPPORTED_FORMAT: &str = "unsupported wav format";
const ERR_BROKEN_WAV: &str = "broken wav file";
const ERR_NOT_LIST_CHUNK: &str = "not a list chunk";
const ERR_FILE_OPEN_ERROR: &str = "failed to open file";
const ERR_UNSUPPORTED_SYSTEM: &str = "incompatible with systems lower than 32-bit";
const ERR_GENERIC_READ_FAIL: &str = "unable to read data";

/// Get header and samples from file
pub fn from_file(file: File) -> Result<WavData, &'static str> {
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
pub fn from_file_str(file_path: &str) -> Result<WavData, &'static str> {
    let f = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Err(ERR_FILE_OPEN_ERROR),
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
    pub fn from_file(file: File) -> Result<Reader, &'static str> {
        let mut data: Vec<u8> = Vec::new();
        let mut f = file;
        match f.read_to_end(&mut data) {
            Ok(_) => {},
            Err(_) => return Err("could not read from file"),
        };
        Self::from_vec(data)
    }
    /// Crate Reader Object from Vec
    pub fn from_vec(data: Vec<u8>) -> Result<Reader, &'static str> {
        let reader = Reader {
            cur: Cursor::new(data),
            header: None,
        };
        Ok(reader)
    }
    /// Read Wav file header
    pub fn read_header(&mut self) -> Result<WavHeader, &'static str> {
        let mut header = WavHeader::new();
        // RIFF header
        let riff_tag = self.read_str4();
        if riff_tag != "RIFF" {
            return Err(ERR_INVALID_FORMAT);
        }
        // size
        let chunk_size = self.read_u32().unwrap_or(0);
        if chunk_size < 8 {
            return Err(ERR_INVALID_FORMAT);
        }
        // should be WAVE
        let wave_tag = self.read_str4();
        if  wave_tag != "WAVE" {
            return Err(ERR_INVALID_FORMAT);
        }

        // check for a possible LIST chunk
        // if there is one, skip it
        let _ = self.read_list_chunk();

        // fmt
        let fmt_tag = self.read_str4();
        if fmt_tag != "fmt " {
            return Err(ERR_INVALID_FORMAT);
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
            _ => return Err(ERR_UNSUPPORTED_FORMAT),
        }
        // channels
        let ch = self.read_u16().unwrap_or(0);
        if 1 <= ch && ch <= 2 {
            header.channels = ch;
        }
        // sample_rate
        header.sample_rate = self.read_u32().unwrap_or(0);
        if header.sample_rate < 32 {
            return Err(ERR_INVALID_FORMAT);
        }
        // ave bytes per sec (sample_rate * bits * channels)
        let bytes_per_sec = self.read_u32().unwrap_or(0);
        if bytes_per_sec < 8 {
            return Err(ERR_BROKEN_WAV);
        }
        // nBlockAlign (channels * bits  / 8)
        let _data_block_size = self.read_u16().unwrap_or(0);
        // Bits per sample
        let bits_per_sample = self.read_u16().unwrap_or(0);
        if bits_per_sample < 8 {
            return Err(ERR_BROKEN_WAV);
        }
        header.bits_per_sample = bits_per_sample;
        // println!("chunk_size={}",chunk_size);
        let pos = self.cur.position() + chunk_size as u64 - 16;
        self.cur.set_position(pos);

        // check for a possible LIST chunk
        // if there is one, skip it
        let _ = self.read_list_chunk();

        // set to header
        self.header = Some(header);
        Ok(header)
    }

    /// Read a LIST chunk
    /// This function will only progres the internal data cursor when `Ok()` is returned
    /// In the case of `Err()`, the cursor will not have moved
    pub fn read_list_chunk(&mut self) -> Result<Vec<u8>, &'static str> {
        // keep track of the position, in case we error, we can jump back
        let begin_position = self.cur.position();

        // check the tag
        let info_tag = self.read_str4();
        if info_tag != "LIST" {
            self.cur.set_position(begin_position);
            return Err(ERR_NOT_LIST_CHUNK);
        }
        // retrieve the info size and convert to an usize
        let Some(read_size) = self.read_u32() else {
            self.cur.set_position(begin_position);
            return Err(ERR_INVALID_FORMAT)
        };
        let Ok(read_size) = read_size.try_into() else {
            self.cur.set_position(begin_position);
            return Err(ERR_UNSUPPORTED_SYSTEM)
        };

        // read the data and return it
        let mut data = vec![0; read_size];
        match self.cur.read_exact(&mut data) {
            Ok(_) => Ok(data),
            Err(_) => {
                self.cur.set_position(begin_position);
                Err(ERR_GENERIC_READ_FAIL)
            }
        }
    }

    pub fn get_samples_f32(&mut self) -> Result<Vec<f32>, &'static str> {
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
            let h = &self.header.unwrap();
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
                        _ => return Err(ERR_UNSUPPORTED_FORMAT),
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
                        _ => return Err(ERR_UNSUPPORTED_FORMAT),
                    }
                },
                _ => return Err(ERR_UNSUPPORTED_FORMAT),
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

