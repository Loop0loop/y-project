#[derive(Debug, Clone, Copy)]
pub(super) struct RenderViewport {
    pub(super) pixel_width: u32,
    pub(super) pixel_height: u32,
    pub(super) cell_rows: u16,
    pub(super) offset_x: u16,
    pub(super) offset_y: u16,
}

impl RenderViewport {
    pub(super) fn new(cols: u16, rows: u16, max_render_cells: Option<u32>) -> Self {
        let (render_cols, render_rows) =
            cap_cells(u32::from(cols), u32::from(rows), max_render_cells);
        Self {
            pixel_width: render_cols,
            pixel_height: render_rows * 2,
            cell_rows: render_rows as u16,
            offset_x: ((u32::from(cols) - render_cols) / 2) as u16,
            offset_y: ((u32::from(rows) - render_rows) / 2) as u16,
        }
    }

    pub(super) fn frame_len(self) -> usize {
        (self.pixel_width * self.pixel_height * 3) as usize
    }
}

pub(super) fn cap_cells(cols: u32, rows: u32, max_render_cells: Option<u32>) -> (u32, u32) {
    let Some(max_render_cells) = max_render_cells else {
        return (cols.max(1), rows.max(1));
    };
    let cells = cols.saturating_mul(rows).max(1);
    if cells <= max_render_cells {
        return (cols.max(1), rows.max(1));
    }

    let scale = (max_render_cells as f64 / cells as f64).sqrt();
    (
        ((cols as f64 * scale).floor() as u32).max(1),
        ((rows as f64 * scale).floor() as u32).max(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::video::config::MAX_RENDER_CELLS;

    #[test]
    fn caps_large_viewport_to_render_budget() {
        let (cols, rows) = cap_cells(240, 80, Some(MAX_RENDER_CELLS));
        assert!(cols * rows <= MAX_RENDER_CELLS);
    }

    #[test]
    fn small_viewport_uses_full_terminal_resolution() {
        assert_eq!(cap_cells(100, 40, Some(MAX_RENDER_CELLS)), (100, 40));
    }

    #[test]
    fn uncapped_viewport_uses_full_terminal_resolution() {
        assert_eq!(cap_cells(240, 80, None), (240, 80));
    }
}
