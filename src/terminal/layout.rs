use super::metrics::{TerminalGrid, TerminalPixels};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CellRect {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub(crate) fn print_layout_fixture() {
    let grid = TerminalGrid {
        cols: 213,
        rows: 60,
    };
    let pixels = TerminalPixels {
        width: 3024,
        height: 1964,
    };
    let rect = CellRect {
        x: 10,
        y: 5,
        width: 80,
        height: 20,
    };
    let (x, y, width, height) = rect_to_pixels(grid, pixels, rect);

    println!("grid={}x{}", grid.cols, grid.rows);
    println!("pixels={}x{}", pixels.width, pixels.height);
    println!(
        "rect=x:{},y:{},w:{},h:{}",
        rect.x, rect.y, rect.width, rect.height
    );
    println!("bbox=x:{},y:{},w:{},h:{}", x, y, width, height);
}

pub(crate) fn rect_to_pixels(
    grid: TerminalGrid,
    pixels: TerminalPixels,
    rect: CellRect,
) -> (u32, u32, u32, u32) {
    let cell_w = f64::from(pixels.width) / f64::from(grid.cols);
    let cell_h = f64::from(pixels.height) / f64::from(grid.rows);

    let x = (f64::from(rect.x) * cell_w).floor() as u32;
    let y = (f64::from(rect.y) * cell_h).floor() as u32;
    let width = (f64::from(rect.width) * cell_w).round() as u32;
    let height = (f64::from(rect.height) * cell_h).round() as u32;

    (
        x,
        y,
        width.min(u32::from(pixels.width).saturating_sub(x)),
        height.min(u32::from(pixels.height).saturating_sub(y)),
    )
}

#[cfg(test)]
fn rect_to_pixels_struct(grid: TerminalGrid, pixels: TerminalPixels, rect: CellRect) -> PixelRect {
    let (x, y, width, height) = rect_to_pixels(grid, pixels, rect);
    PixelRect {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_cell_rect_to_pixel_rect_with_fractional_cells() {
        let bbox = rect_to_pixels_struct(
            TerminalGrid {
                cols: 213,
                rows: 60,
            },
            TerminalPixels {
                width: 3024,
                height: 1964,
            },
            CellRect {
                x: 10,
                y: 5,
                width: 80,
                height: 20,
            },
        );

        assert_eq!(
            bbox,
            PixelRect {
                x: 141,
                y: 163,
                width: 1136,
                height: 655,
            }
        );
    }

    #[test]
    fn clamps_pixel_rect_to_terminal_bounds() {
        let bbox = rect_to_pixels_struct(
            TerminalGrid { cols: 10, rows: 10 },
            TerminalPixels {
                width: 101,
                height: 101,
            },
            CellRect {
                x: 9,
                y: 9,
                width: 5,
                height: 5,
            },
        );

        assert_eq!(
            bbox,
            PixelRect {
                x: 90,
                y: 90,
                width: 11,
                height: 11,
            }
        );
    }
}
