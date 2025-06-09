pub struct Converter;

impl Converter {
    pub fn round_f64(value: f64, decimal_places: i32) -> f64 {
        let multiplier = 10.0_f64.powi(decimal_places);
        (value * multiplier).round() / multiplier
    }

    pub fn round_f32(value: f32, decimal_places: i32) -> f32 {
        let multiplier = 10.0_f32.powi(decimal_places);
        (value * multiplier).round() / multiplier
    }
}
