use std::{cell::RefCell, path::Path, process::Command};

use fast_image_resize::{self as fir, FilterType, ResizeAlg};

thread_local! {
    static RESIZER: RefCell<fir::Resizer> = RefCell::new(fir::Resizer::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VideoMetadata {
    pub(super) width: u32,
    pub(super) height: u32,
}

impl VideoMetadata {
    pub(super) fn probe(path: &Path) -> Result<Self, String> {
        if !path.is_file() {
            return Err(format!("video not found: {}", path.display()));
        }

        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("v:0")
            .arg("-show_entries")
            .arg("stream=width,height")
            .arg("-of")
            .arg("csv=p=0:s=x")
            .arg(path)
            .output()
            .map_err(|error| format!("failed to start ffprobe: {error}"))?;
        if !output.status.success() {
            return Err("ffprobe failed to read video metadata".to_string());
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let (width, height) = text
            .trim()
            .split_once('x')
            .ok_or("ffprobe returned invalid video dimensions")?;
        Ok(Self {
            width: width
                .parse()
                .map_err(|_| "invalid ffprobe width".to_string())?,
            height: height
                .parse()
                .map_err(|_| "invalid ffprobe height".to_string())?,
        })
    }

    pub(super) fn frame_len(self) -> usize {
        (self.width * self.height * 3) as usize
    }
}

pub(super) fn resize_frame(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    dest: &mut Vec<u8>,
    dest_width: u32,
    dest_height: u32,
) -> Result<(), String> {
    dest.resize((dest_width * dest_height * 3) as usize, 0);
    if source_width == dest_width && source_height == dest_height {
        dest.copy_from_slice(source);
        return Ok(());
    }

    const PIXEL: fir::PixelType = fir::PixelType::U8x3;
    let src_img = fir::images::ImageRef::new(source_width, source_height, source, PIXEL)
        .map_err(|error| error.to_string())?;
    let mut dst_img = fir::images::Image::from_slice_u8(dest_width, dest_height, dest, PIXEL)
        .map_err(|error| error.to_string())?;
    let scale =
        (dest_width as f64 / source_width as f64).max(dest_height as f64 / source_height as f64);
    let visible_width = dest_width as f64 / scale;
    let visible_height = dest_height as f64 / scale;
    let left = (source_width as f64 - visible_width) / 2.0;
    let top = (source_height as f64 - visible_height) / 2.0;
    let opts = fir::ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(FilterType::Bilinear))
        .crop(left, top, visible_width, visible_height);

    RESIZER.with(|cell| {
        cell.borrow_mut()
            .resize(&src_img, &mut dst_img, &opts)
            .map_err(|error| error.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_preserves_solid_color() {
        let source = vec![42; 4 * 4 * 3];
        let mut dest = Vec::new();
        resize_frame(&source, 4, 4, &mut dest, 2, 2).unwrap();
        assert_eq!(dest, vec![42; 2 * 2 * 3]);
    }
}
