use image::Rgb;

pub fn gradient_colour(start: Rgb<u8>, end: Rgb<u8>, ratio: f32) -> Rgb<u8> {
    Rgb([
        (start[0] as f32 * (1f32 - ratio) + end[0] as f32 * ratio) as u8,
        (start[1] as f32 * (1f32 - ratio) + end[1] as f32 * ratio) as u8,
        (start[2] as f32 * (1f32 - ratio) + end[2] as f32 * ratio) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: Rgb<u8> = Rgb([255u8, 255u8, 255u8]);
    const BLUE: Rgb<u8> = Rgb([0u8, 0u8, 255u8]);

    #[test]
    fn check_gradient_zero() {
        assert_eq!(gradient_colour(WHITE, BLUE, 0.0), WHITE);
    }

    #[test]
    fn check_gradient_one() {
        assert_eq!(gradient_colour(WHITE, BLUE, 1.0), BLUE);
    }

    #[test]
    fn check_gradient_half() {
        assert_eq!(gradient_colour(WHITE, BLUE, 0.5), Rgb([127, 127, 255]));
    }
}
