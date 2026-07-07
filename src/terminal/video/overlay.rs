use std::path::{Path, PathBuf};

pub(super) struct SplashOverlay {
    path: Option<PathBuf>,
    size: Option<(u32, u32)>,
    rgba: Vec<u8>,
}

impl SplashOverlay {
    pub(super) fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            size: None,
            rgba: Vec::new(),
        }
    }

    pub(super) fn blend_into(
        &mut self,
        rgb: &mut [u8],
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let Some(path) = self.path.clone() else {
            return Ok(());
        };
        self.ensure_rendered(&path, width, height)?;
        blend_premul_rgba(rgb, &self.rgba);
        Ok(())
    }

    fn ensure_rendered(&mut self, path: &Path, width: u32, height: u32) -> Result<(), String> {
        if self.size == Some((width, height)) {
            return Ok(());
        }

        let svg = std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read overlay SVG {}: {error}", path.display()))?
            .replace("{{WIDTH}}", &width.to_string())
            .replace("{{HEIGHT}}", &height.to_string());
        let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
            .map_err(|error| error.to_string())?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
            .ok_or("failed to allocate overlay pixmap")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        self.rgba = pixmap.data().to_vec();
        self.size = Some((width, height));
        Ok(())
    }
}

fn blend_premul_rgba(rgb: &mut [u8], rgba: &[u8]) {
    for (base, over) in rgb.chunks_exact_mut(3).zip(rgba.chunks_exact(4)) {
        let inv = 255 - u16::from(over[3]);
        base[0] = blend_channel(base[0], over[0], inv);
        base[1] = blend_channel(base[1], over[1], inv);
        base[2] = blend_channel(base[2], over[2], inv);
    }
}

fn blend_channel(base: u8, over: u8, inv_alpha: u16) -> u8 {
    ((u16::from(over) + (u16::from(base) * inv_alpha) / 255).min(255)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blends_premultiplied_overlay() {
        let mut rgb = vec![100, 100, 100];
        blend_premul_rgba(&mut rgb, &[50, 0, 0, 128]);
        assert_eq!(rgb, vec![99, 49, 49]);
    }
}
