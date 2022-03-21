/// Wav header and samples

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

/// join LR chanel to stereo wave
pub fn join_stereo_wave(l_samples: Vec<f32>, r_samples: Vec<f32>) -> Vec<f32> {
    let mut result = Vec::with_capacity(l_samples.len() * 2);
    for i in 0..l_samples.len() {
        let left = l_samples[i];
        let right = r_samples[i];
        result.push(left);
        result.push(right);
    }
    result
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_join_split() {
        let f = vec![0.1, 0.2, 0.3, 0.4];
        let (l, r) = split_stereo_wave(f);
        assert_eq!(l, vec![0.1, 0.3]);
        assert_eq!(r, vec![0.2, 0.4]);
        let lr = join_stereo_wave(l, r);
        assert_eq!(lr, vec![0.1, 0.2, 0.3, 0.4]);
    }
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
