//! Toma's own mark: the app icon (`assets/app-icon/`), decoded once and kept as a texture, for the
//! Home screen and About.

const ICON: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/dev.danielmiguelt.Toma.png");

/// The app icon as a texture.
pub fn mark(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::unique("toma-mark");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(t);
    }
    let img = image::load_from_memory_with_format(ICON, image::ImageFormat::Png).ok()?.to_rgba8();
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    let tex = ctx.load_texture("toma-mark", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    Some(tex)
}

/// Paint the app icon over `r`.
pub fn paint_mark(painter: &egui::Painter, r: egui::Rect) {
    if let Some(tex) = mark(painter.ctx()) {
        painter.image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_icon_decodes() {
        let ctx = egui::Context::default();
        let tex = super::mark(&ctx).expect("the app icon decodes");
        assert_eq!(tex.size(), [256, 256]);
    }
}
