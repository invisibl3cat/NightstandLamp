use image::{self, GenericImageView};
use image::{AnimationDecoder, ImageDecoder};

use std::path::Path;
use std::io::Cursor;

use crate::frame::{FrameSpec, IntoFrameSpec};

fn resample_gif_frames(frame_spec: FrameSpec, frames: image::Frames) -> Vec<image::Frame> {
    let mut resampled_frames = Vec::new();
    for frame in frames {
        let img: image::DynamicImage = match frame {
            Ok(frame) => frame.into_buffer().into(),
            Err(_) => continue,
        };

        let resized_img: image::DynamicImage = image::imageops::resize(&img, frame_spec.width as _, frame_spec.height as _, image::imageops::Nearest).into();
        let resized_frame = image::Frame::new(resized_img.into_rgba8());
        resampled_frames.push(resized_frame);
    }

    resampled_frames
}

fn resample_gif_image(frame_spec: FrameSpec, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let decoder = match image::codecs::gif::GifDecoder::new(Cursor::new(bytes)) {
        Ok(decoder) => decoder,
        Err(e) => return Err(format!("Failed to decode GIF: {}", e)),
    };

    let (w, h) = decoder.dimensions();
    let frames = decoder.into_frames();

    let resampled_frames = if w == frame_spec.width as u32 && h == frame_spec.height as u32 {
        match frames.collect_frames() {
            Ok(frames) => frames,
            Err(e) => return Err(format!("Failed to extract GIF frames: {}", e)),
        }
    } else {
        resample_gif_frames(frame_spec, frames)
    };

    let mut reencoded_img = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(Cursor::new(&mut reencoded_img));
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();

        match encoder.encode_frames(resampled_frames) {
            Ok(()) => (),
            Err(e) => return Err(format!("Failed to encode GIF frames: {}", e)),
        }
    }

    Ok(reencoded_img)
}

fn resample_static_image(frame_spec: FrameSpec, img: image::DynamicImage, img_format: image::ImageFormat) -> Result<Vec<u8>, String> {
    let (w, h) = img.dimensions();

    let resampled_img: image::DynamicImage = if w == frame_spec.width as u32 && h == frame_spec.height as u32 {
        img
    } else {
        image::imageops::resize(&img, frame_spec.width as _, frame_spec.height as _, image::imageops::Nearest).into()
    };

    let mut reencoded_img = Vec::new();
    match resampled_img.write_to(Cursor::new(&mut reencoded_img), img_format) {
        Ok(_) => Ok(reencoded_img),
        Err(e) => Err(format!("Failed to encode image: {}", e)),
    }
}

pub fn is_image(file_path: &Path) -> bool {
    match image::ImageReader::open(file_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn resample_image<F: IntoFrameSpec>(frame_spec: F, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let frame_spec = frame_spec.into_framespec();

    let img = match image::ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
        Ok(img) => img,
        Err(e) => return Err(format!("Failed to determine image format: {}", e)),
    };
    let img_format = match img.format() {
        Some(format) => format,
        None => image::ImageFormat::Png,
    };

    if img_format == image::ImageFormat::Gif {
        resample_gif_image(frame_spec, bytes)
    } else {
        let img = match img.decode() {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to decode image: {}", e)),
        };

        resample_static_image(frame_spec, img, img_format)
    }
}
