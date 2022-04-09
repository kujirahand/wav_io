/// Wav file Splitter
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct WavSplitRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub struct WavSplitOption {
    pub is_debug: bool,
    pub min_silence_level: f32,
    pub min_silence_duration: f32,
    pub min_keep_duration: f32,
    pub margin: f32,
}

impl WavSplitOption {
    pub fn new() -> Self {
        Self {
            is_debug: true,
            min_silence_level: 0.03, // 0.05 / 0.15 / 0.25,
            min_silence_duration: 0.6, // 0.07
            min_keep_duration: 0.02, // 0.01,
            margin: 0.1,
        }
    }
}

/// normalize samples
pub fn normalize_i(samples: &Vec<f32>) -> Vec<i16> {
    let mut result: Vec<i16> = Vec::with_capacity(samples.len());
    let imax = std::i16::MAX as f32;
    // get max value
    let mut max: f32 = 0.0;
    for v in samples.iter() {
        let av = v.abs();
        if av > max { max = av; }
    }
    // mul
    let r = 1.0 / max as f32;
    for i in 0..samples.len() {
        let v = r * samples[i];
        let iv = (v * imax) as i16;
        result.push(iv);
    }
    result
}

pub fn normalize_f(samples: &Vec<f32>) -> Vec<f32> {
    let mut result: Vec<f32> = Vec::with_capacity(samples.len());
    let mut max = 0.0;
    for v in samples.iter() {
        let av = v.abs();
        if av > max { max = av; } 
    }
    let r = 1.0 / max;
    for i in 0..samples.len() {
        let v = r * samples[i];
        result.push(v);
    }
    result
}

fn get_max(a: isize, b: isize) -> isize {
    if a > b { a } else { b }
}
fn get_min(a: isize, b: isize) -> isize {
    if a < b { a } else { b }
}

/// split wave data
pub fn split_samples(samples: &mut Vec<f32>, sample_rate: u32, opt: &WavSplitOption) -> Vec<WavSplitRange> {
    let silence_thresh = opt.min_silence_level;
    let silence_thresh_i = (silence_thresh * std::i16::MAX as f32) as i16;
    let min_silence_len = (sample_rate as f32 * opt.min_silence_duration) as usize;
    let min_keep_len = (sample_rate as f32 * opt.min_keep_duration) as usize;
    let margin_len = (sample_rate as f32 * opt.margin) as usize;
    let mut result = vec![];
    let samples_len = samples.len();
    let mut rms_size: u32 = (sample_rate / 300) as u32;
    if rms_size < 5 { rms_size = 5; }
    if opt.is_debug {
        println!("silence_thresh={}", silence_thresh);
        println!("min_silence_len={}", min_silence_len);
        println!("min_keep_len={}", min_keep_len);
        println!("rms_size={}", rms_size);
    }
    if samples_len == 0 { return result; }
    // normalize
    let samples = normalize_i(samples);
    if opt.is_debug {
        println!("normalized={}", samples.len());
    }
    // detect silence
    let mut th_vec:Vec<i16> = Vec::with_capacity(samples_len);
    let mut remain :usize = 0;
    let v_on = std::i16::MAX / 2;
    let v_off = 0;
    for i in 0..samples_len {
        if remain > 0 {
            th_vec.push(v_on);
            remain -= 1;
            continue;
        }
        let rms = calc_rms(&samples, i, rms_size as usize);
        if rms > silence_thresh_i {
            remain = min_keep_len;
            th_vec.push(v_on);
            continue;
        }
        th_vec.push(v_off);
    }

    // detect silence
    let mut si_vec:Vec<(usize,usize)> = vec![];
    let mut status: i16 = v_off;
    let mut last: usize = 0;
    for i in 0..samples_len {
        let v = th_vec[i];
        if status == v { // same as last value
            continue;
        }
        status = v;
        if v == v_off {
            last = i;
            continue;
        }
        let si = (last, i);
        let len = si.1 - si.0;
        if len > min_silence_len {
            si_vec.push(si);
        }
    }
    if samples_len - last > min_silence_len {
        si_vec.push((last, samples_len));
    }
    if opt.is_debug {
        println!("silence={:?}", si_vec);
    }
    if si_vec.len() == 0 { return result; }

    // reverse silence
    let mut last = 0;
    for r in si_vec.iter() {
        let len = r.0 as isize - last as isize;
        if len < min_keep_len as isize { continue; }
        let res = WavSplitRange{start: last, end: r.1};
        result.push(res);
        last = r.1;
    }
    let len = samples_len - last;
    if len > min_keep_len {
        result.push(WavSplitRange{start: last, end: samples_len});
    }

    // margin
    for i in 0.. result.len() {
        result[i].start = get_max(0, result[i].start as isize - margin_len as isize) as usize;
    }

    // result
    if opt.is_debug {
        println!("{:?}", result);
    }
    result
}

pub fn vec_sq(samples: &mut Vec<i16>) {
    for i in 0..samples.len() {
        let v = samples[i];
        let v2 = v.wrapping_mul(v);
        samples[i] = v2.abs();
    }
}

#[allow(dead_code)]
pub fn calc_rms2_prepare(samples: &Vec<i32>) -> Vec<i32> {
    let mut result: Vec<i32> = Vec::with_capacity(samples.len());
    for i in 0..samples.len() {
        let v:i32 = samples[i];
        let vv:i32 = v.wrapping_mul(v);
        result.push(vv);
    }
    result
}

#[allow(dead_code)]
fn calc_rms2_i(samples: &Vec<i32>, start:usize, size: usize) -> f32 {
    let last = start + size;
    let mut total: isize = 0;
    for i in start..last {
        total += samples[i] as isize;
    }
    if total == 0 { return 0.0; }
    ((total as usize / size) as f32).sqrt()
}

#[allow(dead_code)]
fn calc_rms(samples: &Vec<i16>, pos:usize, size: usize) -> i16 {
    let samples_len = samples.len();
    let end = get_min((pos + size) as isize, samples_len as isize) as usize;
    let start = pos;
    let size_act = end - start + 1;
    let mut total: isize = 0;
    for i in start..end {
        let v = samples[i] as isize;
        let v2 = v * v;
        total += v2;
    }
    ((total / size_act as isize) as f32).sqrt() as i16
}


pub fn sub_samples(samples: &Vec<f32>, range: WavSplitRange) -> Vec<f32> {
    let mut result = Vec::with_capacity(range.end - range.start);
    for i in range.start..range.end {
        let v = samples[i];
        result.push(v);
    }
    result
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize() {
        let f1 = vec![0.0, 0.5, 0.0, 0.5];
        let f2 = normalize_i(&f1);
        assert_eq!(f2, vec![0, i16::MAX, 0, i16::MAX]);
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
*/
