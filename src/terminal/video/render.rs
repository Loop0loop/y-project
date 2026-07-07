use super::{config::VideoRenderMode, viewport::RenderViewport};

const ASCII_CHARS: &[u8] = b" .:-=+*#%@";

pub(super) fn render_frame(
    mode: VideoRenderMode,
    frame: &[u8],
    viewport: RenderViewport,
    out: &mut Vec<u8>,
) {
    match mode {
        VideoRenderMode::Ascii => render_ascii_frame(frame, viewport, out),
        VideoRenderMode::Rgb => render_rgb_frame(frame, viewport, out),
    }
}

fn render_ascii_frame(frame: &[u8], viewport: RenderViewport, out: &mut Vec<u8>) {
    out.clear();
    out.extend_from_slice(b"\x1b[?2026h");
    let width = viewport.pixel_width as usize;

    for row in 0..usize::from(viewport.cell_rows) {
        push_cursor(out, viewport.offset_x, viewport.offset_y + row as u16);
        for x in 0..width {
            let top = luma(frame, width, x, row * 2);
            let bottom = luma(frame, width, x, row * 2 + 1);
            out.push(ascii_for_luma((top + bottom) / 2));
        }
    }
    out.extend_from_slice(b"\x1b[?2026l");
}

fn render_rgb_frame(frame: &[u8], viewport: RenderViewport, out: &mut Vec<u8>) {
    out.clear();
    out.extend_from_slice(b"\x1b[?2026h");
    let width = viewport.pixel_width as usize;
    let mut last_fg = None;
    let mut last_bg = None;

    for row in 0..usize::from(viewport.cell_rows) {
        push_cursor(out, viewport.offset_x, viewport.offset_y + row as u16);
        for x in 0..width {
            let fg = rgb_at(frame, width, x, row * 2);
            let bg = rgb_at(frame, width, x, row * 2 + 1);
            push_color(out, fg, bg, &mut last_fg, &mut last_bg);
            out.extend_from_slice("▀".as_bytes());
        }
        out.extend_from_slice(b"\x1b[0m");
        last_fg = None;
        last_bg = None;
    }
    out.extend_from_slice(b"\x1b[?2026l");
}

fn push_cursor(out: &mut Vec<u8>, x: u16, y: u16) {
    out.extend_from_slice(b"\x1b[");
    push_u16(out, y + 1);
    out.push(b';');
    push_u16(out, x + 1);
    out.push(b'H');
}

fn rgb_at(frame: &[u8], width: usize, x: usize, y: usize) -> (u8, u8, u8) {
    let offset = (y * width + x) * 3;
    if offset + 2 >= frame.len() {
        return (0, 0, 0);
    }
    (frame[offset], frame[offset + 1], frame[offset + 2])
}

fn push_color(
    out: &mut Vec<u8>,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    last_fg: &mut Option<(u8, u8, u8)>,
    last_bg: &mut Option<(u8, u8, u8)>,
) {
    let fg_changed = Some(fg) != *last_fg;
    let bg_changed = Some(bg) != *last_bg;
    if !fg_changed && !bg_changed {
        return;
    }

    out.extend_from_slice(b"\x1b[");
    if fg_changed {
        push_rgb_code(out, b"38;2;", fg);
    }
    if fg_changed && bg_changed {
        out.push(b';');
    }
    if bg_changed {
        push_rgb_code(out, b"48;2;", bg);
    }
    out.push(b'm');
    *last_fg = Some(fg);
    *last_bg = Some(bg);
}

fn push_rgb_code(out: &mut Vec<u8>, prefix: &[u8], rgb: (u8, u8, u8)) {
    out.extend_from_slice(prefix);
    push_u8(out, rgb.0);
    out.push(b';');
    push_u8(out, rgb.1);
    out.push(b';');
    push_u8(out, rgb.2);
}

fn push_u8(out: &mut Vec<u8>, value: u8) {
    if value >= 100 {
        out.push(b'0' + value / 100);
        out.push(b'0' + (value / 10) % 10);
        out.push(b'0' + value % 10);
    } else if value >= 10 {
        out.push(b'0' + value / 10);
        out.push(b'0' + value % 10);
    } else {
        out.push(b'0' + value);
    }
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    for byte in value.to_string().bytes() {
        out.push(byte);
    }
}

fn luma(frame: &[u8], width: usize, x: usize, y: usize) -> u32 {
    let offset = (y * width + x) * 3;
    if offset + 2 >= frame.len() {
        return 0;
    }
    (u32::from(frame[offset]) * 299
        + u32::from(frame[offset + 1]) * 587
        + u32::from(frame[offset + 2]) * 114)
        / 1000
}

fn ascii_for_luma(luma: u32) -> u8 {
    let index = (luma.min(255) as usize * (ASCII_CHARS.len() - 1)) / 255;
    ASCII_CHARS[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport(width: u32) -> RenderViewport {
        RenderViewport {
            pixel_width: width,
            pixel_height: 2,
            cell_rows: 1,
            offset_x: 0,
            offset_y: 0,
        }
    }

    #[test]
    fn maps_luma_to_ascii_density() {
        assert_eq!(ascii_for_luma(0), b' ');
        assert_eq!(ascii_for_luma(255), b'@');
    }

    #[test]
    fn renders_two_pixel_rows_as_one_ascii_row() {
        let frame = vec![
            0, 0, 0, 255, 255, 255, //
            0, 0, 0, 255, 255, 255,
        ];
        let mut out = Vec::new();
        render_ascii_frame(&frame, viewport(2), &mut out);
        assert_eq!(out, b"\x1b[?2026h\x1b[1;1H @\x1b[?2026l");
    }

    #[test]
    fn renders_rgb_half_block_cells() {
        let frame = vec![1, 2, 3, 4, 5, 6];
        let mut out = Vec::new();
        render_rgb_frame(&frame, viewport(1), &mut out);
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "\x1b[?2026h\x1b[1;1H\x1b[38;2;1;2;3;48;2;4;5;6m▀\x1b[0m\x1b[?2026l"
        );
    }
}
