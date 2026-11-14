//! Import mode, which is Toma's Home (the app bar's Home and "Import"): the creator suite's Home
//! screen (`nodalia_theme::creator::home`, the same as Lienzo's and Pluma's). The rail holds New
//! project, Open project…, Home and the links; the page the recent projects and the sources to
//! import media from.

use egui::{Align2, Rect, Sense, pos2, vec2};
use nodalia_theme::creator::home;
use serde_json::json;

use crate::TomaApp;

pub fn show(app: &mut TomaApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let look = t.creator();
    let ctx = ui.ctx().clone();
    let links: Vec<home::Link> = crate::links::ALL
        .iter()
        .filter(|(id, _, _)| matches!(*id, "help.website" | "help.appPage" | "help.github"))
        .map(|(id, label, url)| home::Link { icon: if *id == "help.github" { "code" } else { "globe" }, label: crate::i18n::t(label), url })
        .collect();
    let rail = home::Rail { new_file: tl!("New project"), open: tl!("Open project…"), home: tl!("Home"), learn: None, links: &links };
    let mark = |p: &egui::Painter, r: Rect| crate::brand::paint_mark(p, r);
    let picked = home::view(ui, &look, rect, "Toma", &mark, &rail, |ui| {
        recent(app, ui, &look);
        sources(app, ui, &look);
    });
    match picked {
        Some(home::Pick::NewFile) => {
            let _ = crate::menus::invoke(app, &ctx, "file.newProject", json!({}));
        }
        Some(home::Pick::Open) => {
            let _ = crate::menus::invoke(app, &ctx, "file.open", json!({}));
        }
        Some(home::Pick::Home) => app.ui.mode = crate::state::Mode::Edit,
        Some(home::Pick::Link(i)) => {
            if let Some(link) = links.get(i) {
                let url = link.url.to_string();
                crate::links::open(app, &ctx, &url);
            }
        }
        _ => {}
    }
}

/// The sources Home imports from, as cards: (title, command, what it brings, icon).
fn source_cards() -> [(&'static str, &'static str, &'static str, crate::icons::Icon); 4] {
    use crate::icons::Icon;
    [
        (tl!("Browse files…"), "file.import", tl!("Movies, audio and stills from disk"), Icon::Folder),
        (tl!("Demo footage"), "file.importDemoFootage", tl!("Six procedural 1080p clips with sound"), Icon::Film),
        (tl!("Demo project"), "file.openDemoProject", tl!("A cut sequence with transitions, effects and music"), Icon::Sequence),
        (tl!("Bars and Tone"), "file.newBarsAndTone", tl!("SMPTE HD bars with 1 kHz reference tone"), Icon::Grid),
    ]
}

/// Import media: a card per source; a click runs it and goes to Edit. Drops anywhere in the
/// window import too.
fn sources(app: &mut TomaApp, ui: &mut egui::Ui, look: &nodalia_theme::creator::Look) {
    let t = app.tokens;
    home::section_heading(ui, look, tl!("Import media"));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(tl!("Add media to your project. Drop files or folders anywhere in the window, or choose a source below."))
            .color(t.text_dim)
            .size(12.5),
    );
    ui.add_space(12.0);
    let ctx = ui.ctx().clone();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
        for (title, cmd, sub, icon) in source_cards() {
            let (r, resp) = ui.allocate_exact_size(vec2(220.0, 132.0), Sense::click());
            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            app.auto.add(&format!("import.{cmd}"), r, title);
            let p = ui.painter();
            p.rect(r, 12.0, if resp.hovered() { t.hover } else { t.field_bg }, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
            let disc = Rect::from_min_size(r.min + vec2(16.0, 16.0), vec2(40.0, 40.0));
            p.rect_filled(disc, 10.0, t.accent.gamma_multiply(0.16));
            crate::icons::paint(p, disc.shrink(9.0), icon, t.hot_text);
            p.text(r.min + vec2(16.0, 84.0), Align2::LEFT_CENTER, title, crate::theme::Tokens::semibold(14.0), t.text);
            let sub_rect = Rect::from_min_max(r.min + vec2(16.0, 96.0), r.max - vec2(12.0, 8.0));
            let mut job = egui::text::LayoutJob::simple(sub.to_string(), crate::theme::Tokens::ui(11.5), t.text_dim, sub_rect.width());
            job.wrap.max_rows = 2;
            let g = p.layout_job(job);
            p.galley(sub_rect.min, g, t.text_dim);
            if resp.clicked() {
                let _ = crate::menus::invoke(app, &ctx, cmd, json!({}));
                app.ui.mode = crate::state::Mode::Edit;
            }
        }
    });
}

/// Recent projects listed on Home, as in File ▸ Open Recent.
const RECENT_ROWS: usize = 6;
const RECENT_ROW_H: f32 = 34.0;

/// Recent Projects (newest first, file name with its folder beside it); a click opens one. Nothing
/// when there are none.
fn recent(app: &mut TomaApp, ui: &mut egui::Ui, look: &nodalia_theme::creator::Look) {
    let paths: Vec<String> = app.session.prefs.general.recent_projects.iter().take(RECENT_ROWS).cloned().collect();
    if paths.is_empty() {
        return;
    }
    let t = app.tokens;
    home::section_heading(ui, look, tl!("Recent projects"));
    ui.add_space(10.0);
    let mut open = None;
    let width = ui.available_width().min(640.0);
    for (i, path) in paths.iter().enumerate() {
        let (row, resp) = ui.allocate_exact_size(vec2(width, RECENT_ROW_H), Sense::click());
        let name = crate::menus::recent_name(path);
        let resp = resp.on_hover_text(path.as_str()).on_hover_cursor(egui::CursorIcon::PointingHand);
        app.auto.add(&format!("import.recent.{i}"), row, &name);
        if resp.hovered() {
            ui.painter().rect_filled(row, 10.0, t.hover);
        }
        let p = ui.painter().with_clip_rect(row);
        crate::icons::paint(&p, Rect::from_center_size(pos2(row.min.x + 18.0, row.center().y), vec2(16.0, 16.0)), crate::icons::Icon::Sequence, t.text_dim);
        let name_r = p.text(pos2(row.min.x + 38.0, row.center().y), Align2::LEFT_CENTER, &name, crate::theme::Tokens::ui(13.0), t.text);
        if let Some(folder) = std::path::Path::new(path).parent() {
            p.text(pos2(name_r.max.x + 12.0, row.center().y), Align2::LEFT_CENTER, folder.to_string_lossy(), crate::theme::Tokens::ui(11.5), t.text_dim);
        }
        if resp.clicked() {
            open = Some(path.clone());
        }
    }
    ui.add_space(30.0);
    if let Some(path) = open {
        let ctx = ui.ctx().clone();
        // unsaved changes: Save / Don't Save / Cancel first, asked over the project they were made in;
        // Home switches to Edit once the chosen project opens, and stays after Cancel
        if crate::panels::clip_dialogs::ask_before_open(app, &path).is_some() {
            if let Some(info) = app.ui.clip_dialog.as_mut().and_then(|d| d.info.as_object_mut()) {
                info.insert("editAfterOpen".into(), json!(true));
            }
        } else if crate::menus::invoke(app, &ctx, "file.open", json!({"path": path})).is_ok() {
            app.ui.mode = crate::state::Mode::Edit;
        }
    }
}
