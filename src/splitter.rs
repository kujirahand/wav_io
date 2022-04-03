/// Wav file Splitter

#[derive(Debug,Copy, Clone,PartialEq)]
pub struct WavSplitRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug,Copy, Clone,PartialEq)]
pub struct WavSplitOption {
    pub use_margin: bool,
    pub margin_sec: f32,
    pub min_silence_level: f32,
    pub min_silence_duration: f32,
    pub min_keep_duration: f32,
}
impl WavSplitOption {
    pub fn new() -> Self {
        Self {
            use_margin: true,
            margin_sec: 0.01, // 0.03
            min_silence_level: 0.001, // 0.3, 0.002, 0.001
            min_silence_duration: 1.4, // 0.4 - 0.7, 1.4, 1.8
            min_keep_duration: 0.3,
        }
    }
}


/// normalize samples
pub fn normalize(samples: &mut Vec<f32>) {
    let mut max = 0.0;
    for v in samples.iter() {
        let av = v.abs();
        if av > max { max = av; } 
    }
    let r = 1.0 / max;
    for i in 0..samples.len() {
        samples[i] = r * samples[i];
    }
}

fn get_max(a: isize, b: isize) -> isize {
    if a > b { a } else { b }
}

/// Status for Splitter
#[derive(Debug,Clone,PartialEq)]
enum SplitStatus {
    Silence,
    LoudSound,
    FindingSilence,
}

/// split wave data
pub fn split_samples(samples: &mut Vec<f32>, sample_rate: u32, opt: &WavSplitOption) -> Vec<WavSplitRange> {
    let min_silence_level = opt.min_silence_level;
    let min_silence_duration_length = (sample_rate as f32 * opt.min_silence_duration) as usize;
    let min_keep_duration_length = (sample_rate as f32 * opt.min_keep_duration) as usize;

    let mut result_vec = vec![];

    let mut max_val = 0.0;
    let mut silence_start = 0;
    let mut loud_start = 0;

    let check_size = 100;
 
    // append sub samples
    let mut append_sub_smples = |i_begin: usize, i_end: usize| {
        let i_margin = if opt.use_margin {
            (opt.margin_sec * sample_rate as f32).floor() as usize
        } else { 0 };
        // calc margin time
        let i_begin_margin = get_max(0, i_begin as isize - i_margin as isize) as usize;
        let i_end_margin = i_end;
        let new_size = i_end_margin - i_begin_margin + 1;
        if new_size < min_keep_duration_length { return } // Ignore because it is too short.
        // result
        let r = WavSplitRange{start: i_begin_margin, end: i_end_margin};
        result_vec.push(r);
    };

    // normalize
    normalize(samples);
    // calc range sample
    let get_rms = |i:usize, size: usize| -> f32 {
        // let size2 = size / 2;
        let size2 = size;
        let from_i:usize = if i < size2 { 0 } else { i - size2 };
        let mut to_i:usize = from_i + size;
        if samples.len() < to_i { to_i = samples.len(); }
        let mut total = 0.0;
        for j in from_i..to_i {
            let v:f32 = samples[j].clone();
            let v = v.abs();
            total += v;
        }
        total / size as f32
    };

    // check all samples
    let mut status: SplitStatus = SplitStatus::Silence;    
    for (i, v) in samples.iter().enumerate() {
        let av = v.abs();
        if av > max_val { max_val = av; }
        let av = get_rms(i, check_size);
        
        match status {
            SplitStatus::Silence => {
                // silence ended?
                if av > min_silence_level {
                    loud_start = i;
                    status = SplitStatus::LoudSound;
                }
            },
            SplitStatus::LoudSound => {
                // Find silence start
                if av < min_silence_level {
                    silence_start = i;
                    status = SplitStatus::FindingSilence;
                }
            },
            SplitStatus::FindingSilence => {
                // Check silence length
                if av > min_silence_level {
                    let duration = i - silence_start;
                    if duration > min_silence_duration_length {
                        append_sub_smples(loud_start, i - 1);
                        loud_start = i;
                        status = SplitStatus::LoudSound;
                        silence_start = 0;
                    }
                }
            },
        }
    }
    // last wav?
    if (samples.len() - loud_start) > min_keep_duration_length {
        append_sub_smples(loud_start, samples.len() - 1);
    }

    result_vec
}

pub fn sub_samples(samples: &Vec<f32>, range: WavSplitRange) -> Vec<f32> {
    let mut result = Vec::with_capacity(range.end - range.start);
    for i in range.start..=range.end {
        let v = samples[i];
        result.push(v);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize() {
        let mut f1 = vec![0.2, 0.6, 2.0];
        normalize(&mut f1);
        assert_eq!(f1, vec![0.1, 0.3, 1.0]);
    }

    #[test]
    fn test_split1() {
        let mut samples = vec![0.0,0.0,0.0, 1.0,0.9,0.8];
        let res = split_samples(&mut samples, 3, &WavSplitOption::new());
        assert_eq!(res.len(), 1);
        let part = sub_samples(&samples, res[0]);
        assert_eq!(part, vec![1.0,0.9,0.8]);
    }

    #[test]
    fn test_split2() {
        let mut samples = vec![0.0,0.0,0.0, 0.8,0.8,0.8, 0.0,0.0,0.0, 0.8,0.8,0.4];
        let res = split_samples(&mut samples, 3, &WavSplitOption::new());
        assert_eq!(res.len(), 2);
        if res.len() >= 2 {
            let part = sub_samples(&samples, res[0]);
            assert_eq!(part, vec![1.0, 1.0, 1.0, 0.0,0.0,0.0]);
            let part = sub_samples(&samples, res[1]);
            assert_eq!(part, vec![1.0, 1.0, 0.5]);
        }
    }

    #[test]
    fn test_split3() {
        let mut samples = vec![0.0, 0.8,0.8,0.8, 0.0,0.0, 0.8,0.8,0.4];
        let res = split_samples(&mut samples, 3, &WavSplitOption::new());
        assert_eq!(res.len(), 2);
        if res.len() >= 2 {
            let part = sub_samples(&samples, res[0]);
            assert_eq!(part, vec![1.0, 1.0, 1.0, 0.0,0.0]);
            let part = sub_samples(&samples, res[1]);
            assert_eq!(part, vec![1.0, 1.0, 0.5]);
        }
    }
}
