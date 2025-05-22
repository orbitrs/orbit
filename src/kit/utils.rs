// Utility functions for OrbitKit

/// Color utilities
pub mod color {
    /// Lighten a color by a percentage
    pub fn lighten(color: &str, _amount: f32) -> String {
        // For now, just a placeholder
        // In a real implementation, this would lighten the color
        color.to_string()
    }

    /// Darken a color by a percentage
    pub fn darken(color: &str, _amount: f32) -> String {
        // For now, just a placeholder
        // In a real implementation, this would darken the color
        color.to_string()
    }

    /// Convert RGB to HSL
    pub fn rgb_to_hsl(_r: u8, _g: u8, _b: u8) -> (f32, f32, f32) {
        // For now, just a placeholder
        // In a real implementation, this would convert RGB to HSL
        (0.0, 0.0, 0.0)
    }

    /// Convert HSL to RGB
    pub fn hsl_to_rgb(_h: f32, _s: f32, _l: f32) -> (u8, u8, u8) {
        // For now, just a placeholder
        // In a real implementation, this would convert HSL to RGB
        (0, 0, 0)
    }
}

/// String utilities
pub mod string {
    /// Truncate a string to a maximum length
    pub fn truncate(s: &str, max_length: usize) -> String {
        if s.len() <= max_length {
            s.to_string()
        } else {
            format!("{}...", &s[0..max_length - 3])
        }
    }

    /// Capitalize the first letter of a string
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                let first_upper = first.to_uppercase().to_string();
                first_upper + chars.as_str()
            }
        }
    }
}

/// Math utilities
pub mod math {
    /// Clamp a value between a minimum and maximum
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Linear interpolation between two values
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}
