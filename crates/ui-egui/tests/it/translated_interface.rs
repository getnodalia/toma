//! Every screen draws its text in the interface language. The dialog or panel behind every menu
//! command is drawn in Spanish; a text that is still the English key of a Spanish catalog row was
//! drawn without going through the catalog.
use std::collections::{BTreeMap, BTreeSet};

use egui::epaint::Shape;
use egui_kittest::Harness;
use serde_json::json;
use toma_engine::Session;
use toma_ui_egui::TomaApp;
use toma_ui_egui::i18n::Language;
use toma_ui_egui::menus;

/// Names of what the app ships or the walk makes, drawn as stored: a project, a sequence, a
/// bin, a clip, a preset or a label is data the person can rename, so it is not interface text.
const DATA: &[&str] = &[
    // Bins, projects and clips the app makes or ships.
    "Bars and Tone",
    "Effects",
    "Graphics",
    "Music",
    "Sequences",
    "Titles",
    "Untitled",
    // The label colours' names: Settings ▸ Labels lets the person rewrite them.
    "Blue",
    "Brown",
    "Caribbean",
    "Cerulean",
    "Forest",
    "Green",
    "Lavender",
    "Purple",
    "Rose",
    "Tan",
    "Teal",
    "Violet",
    "Yellow",
    // Key names, drawn as the key caps they stand for.
    "Delete",
    "Home",
    "Insert",
    "Left",
    "Right",
];

/// Texts whose draw site is still to be found. Each is a bug, never a place to add to: remove one
/// when its site goes through the catalog.
const KNOWN: &[&str] = &[];

fn harness() -> Harness<'static, TomaApp> {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = TomaApp::new(s);
    app.session.prefs.general.interface_language = "es".into();
    let mut h = Harness::builder().with_size(egui::vec2(1600.0, 980.0)).with_max_steps(10_000).allow_missing_glyphs().build_eframe(move |_cc| app);
    h.run_steps(4);
    h
}

fn collect_shape(shape: &Shape, out: &mut BTreeMap<String, String>) {
    match shape {
        Shape::Text(t) => {
            for line in t.galley.text().split('\n') {
                let line = line.trim();
                if !line.is_empty() {
                    out.entry(line.to_owned()).or_insert_with(|| "painted".to_owned());
                }
            }
        }
        Shape::Vec(v) => v.iter().for_each(|s| collect_shape(s, out)),
        _ => {}
    }
}

/// Everything the frame paints as text. Accessible names are not read: Toma names its widgets for
/// screen readers in English throughout, which is a separate piece of work from what a sighted
/// person sees.
fn shown(h: &Harness<'_, TomaApp>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for clipped in &h.output().shapes {
        collect_shape(&clipped.shape, &mut out);
    }
    out
}

/// Texts of the frame that are English keys of a Spanish row with another reading.
fn untranslated(frame: &BTreeMap<String, String>) -> Vec<String> {
    let es = Language::Es;
    frame
        .iter()
        .filter(|(t, _)| !DATA.contains(&t.as_str()) && !KNOWN.contains(&t.as_str()))
        .filter(|(t, _)| es.has(t) && es.tr(t) != t.as_str())
        .map(|(t, kind)| format!("{t:?} ({kind})"))
        .collect()
}

/// Commands that leave the app, close the project the walk draws on, or switch the language it checks.
fn skipped(id: &str) -> bool {
    id.starts_with("app.language") || id.contains("quit") || id.contains("exit") || id.starts_with("file.close") || id == "app.quit"
}

fn walk() -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut h = harness();
    let mut note = |what: &str, frame: &BTreeMap<String, String>, failures: &mut Vec<String>| {
        for line in untranslated(frame) {
            if seen.insert(line.clone()) {
                failures.push(format!("{what}: {line}"));
            }
        }
    };
    note("main window", &shown(&h), &mut failures);
    let ids: Vec<String> = menus::menu_items(h.state()).into_iter().filter(|i| i.enabled).map(|i| i.id).collect();
    for id in ids {
        if skipped(&id) {
            continue;
        }
        let ctx = h.ctx.clone();
        let _ = menus::invoke(h.state_mut(), &ctx, &id, json!({}));
        h.run_steps(3);
        note(&format!("command {id}"), &shown(&h), &mut failures);
        for _ in 0..3 {
            h.key_press(egui::Key::Escape);
            h.run_steps(2);
        }
    }
    failures
}

/// A screen that draws a text the catalog translates, without going through the catalog, shows
/// English to everyone who runs Toma in Spanish.
#[test]
fn spanish_interface_has_no_untranslated_text() {
    let failures = walk();
    assert!(failures.is_empty(), "texts drawn in English although Spanish has them:\n{}", failures.join("\n"));
}
