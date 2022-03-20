/// convert stereo to mono
pub fn stereo_to_mono(in_v: Vec<f32>) -> Vec<f32> {
    let new_size = in_v.len() / 2;
    let mut result = Vec::with_capacity(new_size);
    for i in 0..new_size {
        let lv = in_v[i * 2 + 0];
        let rv = in_v[i * 2 + 1];
        result.push((lv + rv) / 2.0);
    }
    result
}

/// convert mono to stereo
pub fn mono_to_stereo(in_v: Vec<f32>) -> Vec<f32> {
    let new_size = in_v.len() * 2;
    let mut result = Vec::with_capacity(new_size);
    for i in 0..in_v.len() {
        let v = in_v[i];
        result.push(v);
        result.push(v);
    }
    result

}

/// split LR channel from stereo wave
pub fn split_stereo_wave(samples: Vec<f32>) -> (Vec<f32>, Vec<f32>) {
    let size = samples.len() / 2;
    let mut l_samples = Vec::with_capacity(size);
    let mut r_samples = Vec::with_capacity(size);
    for i in 0..size {
        l_samples.push(samples[i * 2 + 0]);
        r_samples.push(samples[i * 2 + 1]);
    }
    (l_samples, r_samples)
}

/// resample audio sample rate
pub fn resample(samples: Vec<f32>, cur_rate: u32, new_rate: u32) -> Vec<f32> {
    // same rate
    if cur_rate == new_rate {
        return samples.clone();
    }
    if cur_rate < new_rate {
        resample_upsamle(samples, cur_rate, new_rate)
    } else {
        resample_downsample(samples, cur_rate, new_rate)
    }
}

fn resample_upsamle(samples: Vec<f32>, cur_rate: u32, new_rate: u32) -> Vec<f32> {
    let scale = cur_rate as f32 / new_rate as f32;
    let cur_size = samples.len();
    let new_size = (cur_size as f32 / scale) as usize;
    let mut output = Vec::with_capacity(new_size);
    let mut pos = 0.0;
    for _i in 0..new_size {
        let mut in_pos = pos as usize;
        let mut prop = pos - in_pos as f32;
        if in_pos >= cur_size - 1 {
            in_pos = cur_size - 2;
            prop = 1.0;
        }
        let v = samples[in_pos] * (1.0 - prop) + samples[in_pos + 1] * prop;
        output.push(v);
        pos += scale;
    }
    output
}
fn resample_downsample(samples: Vec<f32>, cur_rate: u32, new_rate: u32) -> Vec<f32> {
    let scale = new_rate as f32 / cur_rate as f32;
    let cur_size = samples.len();
    let new_size = (cur_size as f32 / scale) as usize;
    let mut output = Vec::with_capacity(new_size);
    let mut in_pos = 0;
    let mut out_pos = 0;
    let mut pos = 0.0;
    let mut sum = 0.0;
    while out_pos < new_size {
        if in_pos >= cur_size { break; }
        let val = samples[in_pos];
        in_pos += 1;
        let mut next_pos = pos + scale;
        if next_pos >= 1.0 {
            sum += val * (1.0 - pos);
            output.push(sum);
            next_pos -= 1.0;
            sum = next_pos * val;
        } else {
            sum += scale * val;
        }
        pos = next_pos;
        if (in_pos >= cur_size) && (out_pos < new_size) {
            output.push(sum / pos);
            out_pos += 1;
        }
    }
    output
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stereo_to_mono() {
        let f2 = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0];
        let f1 = stereo_to_mono(f2);
        assert_eq!(f1, vec![1.0, 2.0, 3.0]);
    }
    #[test]
    fn test_mono_to_stereo() {
        let f2 = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0];
        let f1 = stereo_to_mono(f2.clone());
        let f2_test = mono_to_stereo(f1);
        assert_eq!(f2, f2_test);
    }
    #[test]
    fn test_split_stereo() {
        let f2 = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0];
        let (f_left, f_right) = split_stereo_wave(f2);
        assert_eq!(f_left, f_right);
    }
}
