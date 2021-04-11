use image::Rgb;

pub fn gradient_colour(start: Rgb<u8>, end: Rgb<u8>, ratio: f32) -> Rgb<u8> {
    Rgb([
        (start[0] as f32 * (1f32 - ratio) + end[0] as f32 * ratio) as u8,
        (start[1] as f32 * (1f32 - ratio) + end[1] as f32 * ratio) as u8,
        (start[2] as f32 * (1f32 - ratio) + end[2] as f32 * ratio) as u8,
    ])
}
