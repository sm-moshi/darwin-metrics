#[derive(Debug, Clone, Copy)]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    pub fn bytes(&self) -> u64 {
        self.0
    }

    pub fn kilobytes(&self) -> f64 {
        self.0 as f64 / 1024.0
    }

    pub fn megabytes(&self) -> f64 {
        self.kilobytes() / 1024.0
    }

    pub fn gigabytes(&self) -> f64 {
        self.megabytes() / 1024.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Percentage(pub f32);

impl Percentage {
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 100.0))
    }

    pub fn value(&self) -> f32 {
        self.0
    }
} 