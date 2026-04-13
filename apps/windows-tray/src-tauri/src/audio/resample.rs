#[derive(Debug, Clone)]
pub struct LinearResampler {
    ratio: f64,
    cursor: f64,
    source: Vec<f32>,
}

impl LinearResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            ratio: f64::from(input_rate) / f64::from(output_rate),
            cursor: 0.0,
            source: Vec::new(),
        }
    }

    pub fn push(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }

        self.source.extend_from_slice(input);
        let mut output = Vec::new();
        while self.cursor + 1.0 < self.source.len() as f64 {
            let index = self.cursor.floor() as usize;
            let frac = (self.cursor - index as f64) as f32;
            let a = self.source[index];
            let b = self.source[index + 1];
            output.push(a + (b - a) * frac);
            self.cursor += self.ratio;
        }

        let consumed = self.cursor.floor() as usize;
        if consumed > 0 {
            self.source.drain(..consumed);
            self.cursor -= consumed as f64;
        }

        output
    }
}
