use std::{env, io::Write, time::Duration};

use super::{
    layout::CellRect,
    session::{TerminalSession, wait_or_interrupt},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KittyImage {
    pub(crate) image_id: u32,
    pub(crate) placement_id: u32,
}

pub(crate) fn run_kitty_demo() -> Result<(), String> {
    if env::var_os("KITTY_WINDOW_ID").is_none() {
        eprintln!(
            "warning: KITTY_WINDOW_ID is not set; non-Kitty terminals should ignore the image payload"
        );
    }

    let image = KittyImage {
        image_id: 424_242,
        placement_id: 7,
    };
    let width = 32;
    let height = 16;
    let rgba = make_demo_rgba(width, height);
    let mut terminal = TerminalSession::enter(false, false)?;
    terminal.register_image(image);

    present_rgba(
        terminal.stdout(),
        &rgba,
        width,
        height,
        CellRect {
            x: 1,
            y: 1,
            width: 12,
            height: 6,
        },
        &image,
        true,
    )?;

    for step in 0..10 {
        present_rgba(
            terminal.stdout(),
            &[],
            width,
            height,
            CellRect {
                x: 1 + step,
                y: 1,
                width: 12,
                height: 6,
            },
            &image,
            false,
        )?;
        write!(
            terminal.stdout(),
            "\x1b[9;2HProject-Y Kitty placement replace {}/10  image_id={} placement_id={}   ",
            step + 1,
            image.image_id,
            image.placement_id
        )
        .map_err(|error| error.to_string())?;
        terminal
            .stdout()
            .flush()
            .map_err(|error| error.to_string())?;
        if wait_or_interrupt(Duration::from_millis(140))? {
            return Ok(());
        }
    }

    wait_or_interrupt(Duration::from_millis(500))?;
    Ok(())
}

pub(crate) fn present_rgba(
    stdout: &mut impl Write,
    rgba: &[u8],
    pixel_width: u32,
    pixel_height: u32,
    cell_rect: CellRect,
    image: &KittyImage,
    transmit_pixels: bool,
) -> Result<(), String> {
    present_rgba_with_z(
        stdout,
        rgba,
        pixel_width,
        pixel_height,
        cell_rect,
        image,
        transmit_pixels,
        0,
    )
}

pub(crate) fn present_rgba_with_z(
    stdout: &mut impl Write,
    rgba: &[u8],
    pixel_width: u32,
    pixel_height: u32,
    cell_rect: CellRect,
    image: &KittyImage,
    transmit_pixels: bool,
    z_index: i32,
) -> Result<(), String> {
    write!(stdout, "\x1b[{};{}H", cell_rect.y + 1, cell_rect.x + 1)
        .map_err(|error| error.to_string())?;
    if transmit_pixels {
        let payload = base64_encode(rgba);
        write!(
            stdout,
            "\x1b_Ga=T,f=32,s={pixel_width},v={pixel_height},i={},p={},c={},r={},C=1,q=1,z={z_index};{payload}\x1b\\",
            image.image_id, image.placement_id, cell_rect.width, cell_rect.height
        )
    } else {
        write!(
            stdout,
            "\x1b_Ga=p,i={},p={},c={},r={},C=1,q=1,z={z_index}\x1b\\",
            image.image_id, image.placement_id, cell_rect.width, cell_rect.height
        )
    }
    .map_err(|error| error.to_string())
}

pub(crate) fn delete_image(stdout: &mut impl Write, image: &KittyImage) -> Result<(), String> {
    write!(
        stdout,
        "\x1b_Ga=d,d=i,i={},p={},q=1\x1b\\",
        image.image_id, image.placement_id
    )
    .map_err(|error| error.to_string())
}

fn make_demo_rgba(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let edge = x == 0 || y == 0 || x == width - 1 || y == height - 1;
            let diagonal = x * height / width == y;
            let (r, g, b) = if edge {
                (255, 240, 90)
            } else if diagonal {
                (0, 220, 255)
            } else {
                (20 + (x * 5) as u8, 40 + (y * 8) as u8, 120)
            };
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    pixels
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::base64_encode;

    #[test]
    fn encodes_base64_with_padding() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }
}
