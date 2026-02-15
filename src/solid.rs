use crate::frame::{IntoFrameSpec, frame_from_rgb};

pub type SmoothSolidColor = Vec<[u8; 3]>;

pub fn make_frame<F: IntoFrameSpec>(frame_spec: F, r: u8, g: u8, b: u8) -> Vec<u8> {
    let frame_spec = frame_spec.into_framespec();

    frame_from_rgb(frame_spec, r, g, b)
}
