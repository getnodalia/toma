//! The app bar (Nodalia's creator suite, 44 pt): the main menu (≡) with About, Preferences,
//! Keyboard Shortcuts and Quit, Home, the Import / Edit / Export modes, then the in-window menus
//! when the desktop has no menu bar of its own, the document title centred, and search, Quick
//! Export and the workspace selector at the right. On Linux, with Nodalia's window controls
//! ([`TomaApp::custom_titlebar`]), the bar is the window's title bar too: its empty space drags
//! the window and the round controls take its right end.

use egui::{Rect, Sense, pos2, vec2};

use crate::TomaApp;
use crate::state::Mode;
use crate::theme::Tokens;

/// Height of the app bar. With `integrated_titlebar` it sits in the macOS title-bar strip.
pub const HEIGHT: f32 = 44.0;
/// A bar button's side.
const BUTTON: f32 = 28.0;

/// A 28 pt icon button of the bar: Nodalia's icon `icon`, a rounded fill under the pointer and
/// while `selected`.
fn icon_button(ui: &mut egui::Ui, t: &Tokens, rect: Rect, icon: &str, id: &str, tip: &str, selected: bool) -> egui::Response {
    let resp = ui.interact(rect, egui::Id::unique(("hdr", id)), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, tip));
    let fill = if selected {
        t.pressed
    } else if resp.hovered() {
        t.hover
    } else {
        egui::Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, nodalia_theme::component::CONTROL_RADIUS, fill);
    if let Some(glyph) = nodalia_theme::icons::get(icon) {
        let ink = if selected || resp.hovered() { t.text } else { t.text_dim };
        nodalia_theme::icons::paint(ui.painter(), rect.shrink(6.0), glyph, nodalia_theme::icons::Style::bold(ink));
    }
    resp
}

/// The main menu's entries: label, command.
const MAIN_MENU: [(&str, &str); 4] =
    [("About Toma", "app.about"), ("Preferences…", "app.settings.general"), ("Keyboard Shortcuts…", "app.keyboardShortcuts"), ("Quit", "app.quit")];

/// Nodalia's main menu: a hamburger at the start of the bar with the app's own entries.
fn main_menu(app: &mut TomaApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let button = icon_button(ui, &t, rect, "menu", "mainMenu", tl!("Main Menu"), false);
    app.auto.add("header.mainMenu", rect, "Main Menu");
    let mut chosen = None;
    egui::Popup::menu(&button).show(|ui| {
        ui.set_min_width(230.0);
        for (i, (label, id)) in MAIN_MENU.iter().enumerate() {
            if i == 3 {
                ui.separator();
            }
            let shortcut = crate::menus::UI_COMMANDS.iter().find(|c| c.id == *id).and_then(|c| c.shortcut).map(crate::menus::shortcut_text).unwrap_or_default();
            let row = ui.add(egui::Button::new(crate::i18n::t(label)).shortcut_text(shortcut).min_size(vec2(0.0, 26.0)));
            app.auto.add(&format!("header.mainMenu.{id}"), row.rect, label);
            if row.clicked() {
                chosen = Some(*id);
                ui.close();
            }
        }
    });
    if let Some(id) = chosen {
        let ctx = ui.ctx().clone();
        if id == "app.quit" {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if let Err(e) = crate::menus::invoke(app, &ctx, id, serde_json::json!({})) {
            app.ui.status = e;
        }
    }
}

/// Draws the app bar in `rect` and handles its clicks.
pub fn show(app: &mut TomaApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let look = t.creator();
    let p = ui.painter().clone();
    // The bar is the window's frame around the islands: window colour, no line under it.
    p.rect_filled(rect, 0.0, t.app_bg);
    if app.custom_titlebar || app.integrated_titlebar {
        crate::titlebar::drag_area(ui, rect);
    }
    let cy = rect.center().y;
    let mut x = rect.min.x + if app.integrated_titlebar { 80.0 } else { crate::theme::ISLAND_GAP + 2.0 };
    let square = |x: f32| Rect::from_min_size(pos2(x, cy - BUTTON / 2.0), vec2(BUTTON, BUTTON));
    main_menu(app, ui, square(x));
    x += BUTTON + 4.0;
    // Home: the Import page, which is Toma's Home screen.
    let home = square(x);
    app.auto.add("header.home", home, "Home");
    if icon_button(ui, &t, home, "home", "home", tl!("Home"), false).on_hover_text(tl!("Home")).clicked() {
        app.ui.mode = Mode::Import;
    }
    x = home.max.x + 10.0;
    // The modes, as the creator suite's tabs.
    let tabs = Rect::from_min_max(pos2(x, rect.min.y), pos2(rect.max.x, rect.max.y));
    let mut tu = ui.new_child(egui::UiBuilder::new().max_rect(tabs).layout(egui::Layout::left_to_right(egui::Align::Center)));
    tu.spacing_mut().item_spacing.x = 2.0;
    for (m, label, shown) in [(Mode::Import, "Import", tl!("Import")), (Mode::Edit, "Edit", tl!("Edit")), (Mode::Export, "Export", tl!("Export"))] {
        let resp = nodalia_theme::creator::tab(&mut tu, &look, shown, app.ui.mode == m);
        app.auto.add(&format!("header.mode.{}", label.to_ascii_lowercase()), resp.rect, label);
        if resp.clicked() {
            app.ui.mode = m;
        }
    }
    let mut left_end = tu.min_rect().max.x;
    // In-window menus when there is no menu bar of the desktop's or the system's.
    if app.ui.show_menu_bar && !app.hooks.global_menu_shown.as_ref().is_some_and(|shown| shown()) {
        let x = left_end + 10.0;
        let menu_rect = Rect::from_min_max(pos2(x, rect.min.y + 8.0), pos2(x + 520.0, rect.max.y - 8.0));
        let mut mu = ui.new_child(egui::UiBuilder::new().max_rect(menu_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
        mu.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        mu.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        mu.style_mut().visuals.override_text_color = Some(t.text_dim);
        crate::menus::menu_bar(app, &mut mu);
        left_end = mu.min_rect().max.x;
    }
    // Right group, from the right: window controls, workspace selector, Quick Export, search.
    let mut rx = rect.max.x - if app.custom_titlebar { crate::titlebar::WIDTH + 8.0 } else { crate::theme::ISLAND_GAP + 2.0 };
    if app.custom_titlebar {
        crate::titlebar::window_controls(app, ui, rect);
    }
    let names = crate::dock::names(&app.workspaces);
    let shown: Vec<String> = names.iter().map(|w| crate::i18n::t(w).to_string()).collect();
    let current = crate::i18n::t(&app.ui.workspace).to_string();
    let ws_w = (p.layout_no_wrap(current.clone(), egui::FontId::proportional(13.0), t.text).size().x + 44.0).clamp(130.0, 200.0);
    let ws_rect = Rect::from_min_size(pos2(rx - ws_w, cy - 15.0), vec2(ws_w, 30.0));
    rx = ws_rect.min.x - 8.0;
    let mut wu = ui.new_child(egui::UiBuilder::new().max_rect(ws_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let saved_from = names.iter().position(|w| !crate::dock::is_builtin(w)).unwrap_or(names.len());
    let refs: Vec<&str> = shown.iter().map(String::as_str).collect();
    let actions = ["reset", "saveChanges", "saveAs", "edit"];
    let action_labels = [tl!("Reset to Saved Layout"), tl!("Save Changes to this Workspace"), tl!("Save as New Workspace…"), tl!("Edit Workspaces…")];
    let pick = nodalia_theme::creator::workspace_selector(&mut wu, &look, "toma", &current, &refs, saved_from, &action_labels, ws_w);
    app.auto.add("header.workspaces", wu.min_rect(), &current);
    // Automation ids for the open list's rows, which the shared selector draws: its clickable
    // widgets that take focus, in order, are the workspaces and then the actions.
    let popup = egui::Id::NULL.with(("ndl_creator_workspace", "toma"));
    if egui::Popup::is_id_open(ui.ctx(), popup) {
        let layer = egui::LayerId::new(egui::Order::Foreground, popup);
        let rows: Vec<Rect> =
            ui.ctx().viewport(|v| v.prev_pass.widgets.get_layer(layer).filter(|w| w.sense.senses_click() && w.sense.is_focusable()).map(|w| w.rect).collect());
        let ids = names.iter().map(|w| format!("header.workspace.{w}")).chain(actions.iter().map(|a| format!("header.workspaces.{a}")));
        for (id, r) in ids.zip(rows) {
            app.auto.add(&id, r, &id);
        }
    }
    match pick {
        Some(nodalia_theme::creator::WorkspacePick::Workspace(i)) => {
            if let Some(w) = names.get(i) {
                app.set_workspace(w);
            }
        }
        Some(nodalia_theme::creator::WorkspacePick::Action(i)) => {
            if let Some(action) = actions.get(i) {
                let ctx = ui.ctx().clone();
                if let Err(e) = crate::menus::invoke(app, &ctx, &format!("window.workspace.{action}"), serde_json::json!({})) {
                    app.ui.status = e;
                }
            }
        }
        None => {}
    }
    // Quick Export: a popup with File Name & Location, a preset list and Export.
    let qe = square(rx - BUTTON);
    rx = qe.min.x - 4.0;
    app.auto.add("header.quickExport", qe, "Quick Export");
    if icon_button(ui, &t, qe, "export", "quickExport", tl!("Quick Export"), app.ui.export.quick_open).on_hover_text(tl!("Quick Export")).clicked() {
        app.ui.export.quick_open = !app.ui.export.quick_open;
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::unique("quick-export-toggled"), true));
    }
    crate::panels::export_mode::quick_export(app, ui.ctx(), pos2((qe.right() - 340.0).max(rect.min.x + 8.0), rect.max.y + 4.0));
    // Search: the Effects panel, where effects and presets are found.
    let search = square(rx - BUTTON);
    rx = search.min.x;
    app.auto.add("header.search", search, "Search");
    if icon_button(ui, &t, search, "search", "search", tl!("Search"), false).on_hover_text(tl!("Search")).clicked() {
        app.show_panel(crate::dock::PanelKind::Effects);
    }
    // The document title, centred while the groups either side leave it room.
    let title = if app.session.is_dirty() { tlf!("{name} - Edited", name = app.session.project.name) } else { app.session.project.name.clone() };
    let (left, right) = (left_end + 12.0, rx - 12.0);
    if right > left + 40.0 {
        let mut job = egui::text::LayoutJob::simple_singleline(title, egui::FontId::new(13.5, egui::FontFamily::Name("medium".into())), t.text);
        job.wrap.max_width = right - left;
        job.wrap.max_rows = 1;
        let galley = p.layout_job(job);
        let half = galley.size().x / 2.0;
        let center = if rect.center().x - half >= left && rect.center().x + half <= right { rect.center().x } else { (left + right) / 2.0 };
        p.with_clip_rect(Rect::from_min_max(pos2(left, rect.min.y), pos2(right, rect.max.y))).galley(
            pos2(center - half, cy - galley.size().y / 2.0),
            galley,
            t.text,
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn workspace_name_uppercasing_is_turkish_aware() {
        crate::i18n::set_current(crate::i18n::Language::Tr);
        assert_eq!(crate::i18n::to_uppercase("Birleştirme"), "BİRLEŞTİRME");
        assert_eq!(crate::i18n::to_uppercase("Altyazılar ve Grafikler"), "ALTYAZILAR VE GRAFİKLER");
        crate::i18n::set_current(crate::i18n::Language::En);
        assert_eq!(crate::i18n::to_uppercase("Birleştirme"), "BIRLEŞTIRME");
    }
}
