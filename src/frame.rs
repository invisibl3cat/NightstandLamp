use image::{AnimationDecoder, Pixel};

use std::io::Cursor;

use crate::imgops;

pub type Frame = Vec<u8>;
pub type Frames = Vec<Frame>;

pub struct FrameSpec {
    pub width: u8,
    pub height: u8,
}
impl FrameSpec {
    pub fn len(&self) -> u32 {
        self.width as u32 * self.height as u32
    }
}

pub trait IntoFrameSpec {
    fn into_framespec(self) -> FrameSpec;
}

impl IntoFrameSpec for (u8, u8) {
    fn into_framespec(self) -> FrameSpec {
        let (width, height) = self;
        FrameSpec{ width, height }
    }
}

impl IntoFrameSpec for FrameSpec {
    fn into_framespec(self) -> FrameSpec {
        self
    }
}

fn image_data_to_frame(img: image::DynamicImage) -> Vec<u8> {
    let mut frame_rows = Vec::new();

    for (idx, row) in img.into_rgba8().rows().enumerate() {
        let mut frame_row = Vec::new();

        let is_reverse_row = idx % 2 == 0;

        for pixel in row {
            let a: f32 = pixel.alpha() as _;
            let rgb = pixel.channels();
            let mut r: f32 = rgb[0] as _;
            let mut g: f32 = rgb[1] as _;
            let mut b: f32 = rgb[2] as _;

            r = r * a / 255.0;
            g = g * a / 255.0;
            b = b * a / 255.0;

            if is_reverse_row {
                frame_row.push(b as u8);
                frame_row.push(g as u8);
                frame_row.push(r as u8);
            } else {
                frame_row.push(r as u8);
                frame_row.push(g as u8);
                frame_row.push(b as u8);
            }
        }

        if is_reverse_row {
            frame_row.reverse();
        }

        frame_rows.push(frame_row);
    }

    let mut frame = Vec::new();
    for frame_row in frame_rows.into_iter().rev() {
        frame.extend(frame_row);
    }

    frame
}

fn frames_from_gif_image(image_bytes: &[u8]) -> Result<Frames, String> {
    let decoder = match image::codecs::gif::GifDecoder::new(Cursor::new(image_bytes)) {
        Ok(decoder) => decoder,
        Err(e) => return Err(format!("Failed to decode GIF: {}", e)),
    };

    let mut frames = Vec::new();
    let gif_frames = decoder.into_frames();
    for gif_frame in gif_frames {
        let img: image::DynamicImage = match gif_frame {
            Ok(gif_frame) => gif_frame.into_buffer().into(),
            Err(_) => continue,
        };

        let frame = image_data_to_frame(img);
        frames.push(frame)
    }

    Ok(frames)
}

fn frames_from_static_image(img: image::DynamicImage) -> Frames {
    let frame = image_data_to_frame(img);

    vec![frame]
}

pub fn frames_from_image<F: IntoFrameSpec>(frame_spec: F, image_bytes: &[u8]) -> Result<Frames, String> {
    let resampled_image = imgops::resample_image(frame_spec, image_bytes)?;

    let img = match image::ImageReader::new(Cursor::new(&resampled_image)).with_guessed_format() {
        Ok(img) => img,
        Err(e) => return Err(format!("Failed to determine image format: {}", e)),
    };
    let img_format = match img.format() {
        Some(format) => format,
        None => image::ImageFormat::Png,
    };


    let frames = if img_format == image::ImageFormat::Gif {
        frames_from_gif_image(&resampled_image)?
    } else {
        let img = match img.decode() {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to decode image: {}", e)),
        };

        frames_from_static_image(img)
    };

    Ok(frames)
}

pub fn frame_from_rgb<F: IntoFrameSpec>(frame_spec: F, r: u8, g: u8, b: u8) -> Vec<u8> {
    let frame_spec = frame_spec.into_framespec();

    let mut frame = Vec::with_capacity(frame_spec.len() as _);
    for _ in 0..frame_spec.width {
        for _ in 0..frame_spec.height {
            frame.push(r);
            frame.push(g);
            frame.push(b);
        }
    }

    frame
}
