//! The Linux desktop's global menu: Toma's menus exported over D-Bus as a `com.canonical.dbusmenu`
//! object and registered with the desktop's `com.canonical.AppMenu.Registrar`, so a top bar that
//! shows the focused app's menus (Nodalix's) shows Toma's. The menus are the in-window bar's
//! ([`menu_items`]); this file only serves them, and a chosen item reaches the app's command inbox
//! as a native menu's does.
//!
//! The registration names this process rather than a window (a Wayland window has no global id),
//! so only a registrar that pairs a menu with its window by process can show it: Nodalix's, whose
//! process also owns `space.unmade.GlobalMenu`. With any other registrar, none, or one that goes
//! away, the menus stay in (or come back to) the window. Keys stay with the window: the shortcuts
//! run as they do without a global menu.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use toma_ui_egui::TomaApp;
use toma_ui_egui::menus::{MENUS, MenuItem as Item, menu_items};
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedValue, StructureBuilder, Value};

const REGISTRAR: &str = "com.canonical.AppMenu.Registrar";
const REGISTRAR_PATH: &str = "/com/canonical/AppMenu/Registrar";
const NODALIX_MENU: &str = "space.unmade.GlobalMenu";
const MENU_PATH: &str = "/com/getnodalia/GlobalMenu";
const MENU_INTERFACE: &str = "com.canonical.dbusmenu";

/// One row of the exported menu.
#[derive(Clone, Debug, PartialEq)]
enum Row {
    Item { id: String, label: String, enabled: bool, checked: Option<bool> },
    Submenu { label: String, children: Vec<i32> },
}

/// The menus as the desktop reads them: rows by id (0 is the root, whose children are the
/// top-level menus).
#[derive(Default)]
struct Shared {
    rows: HashMap<i32, Row>,
    root: Vec<i32>,
    revision: u32,
}

impl Shared {
    /// Number the rows of `items` depth first, from 1, in the in-window bar's order: the menus in
    /// [`MENUS`] order, and a submenu where its first item would be. The same tree always gets the
    /// same ids.
    fn rows_of(items: &[Item]) -> (HashMap<i32, Row>, Vec<i32>) {
        fn level(items: &[&Item], depth: usize, next: &mut i32, rows: &mut HashMap<i32, Row>) -> Vec<i32> {
            let mut ids = Vec::new();
            let mut subs: Vec<&str> = Vec::new();
            for it in items {
                let id = *next;
                let row = if let Some(sub) = it.path.get(depth).map(String::as_str) {
                    if subs.contains(&sub) {
                        continue;
                    }
                    subs.push(sub);
                    *next = next.saturating_add(1);
                    let inner: Vec<&Item> = items.iter().copied().filter(|x| x.path.get(depth).map(String::as_str) == Some(sub)).collect();
                    let children = level(&inner, depth.saturating_add(1), next, rows);
                    Row::Submenu { label: toma_ui_egui::i18n::t(sub).to_string(), children }
                } else {
                    *next = next.saturating_add(1);
                    Row::Item { id: it.id.clone(), label: it.label.clone(), enabled: it.enabled, checked: it.checked }
                };
                rows.insert(id, row);
                ids.push(id);
            }
            ids
        }
        let (mut rows, mut next, mut root) = (HashMap::new(), 1i32, Vec::new());
        for top in MENUS {
            let mine: Vec<&Item> = items.iter().filter(|i| i.path.first().map(String::as_str) == Some(top)).collect();
            if mine.is_empty() {
                continue;
            }
            let id = next;
            next = next.saturating_add(1);
            let children = level(&mine, 1, &mut next, &mut rows);
            rows.insert(id, Row::Submenu { label: toma_ui_egui::i18n::t(top).to_string(), children });
            root.push(id);
        }
        (rows, root)
    }

    fn children(&self, id: i32) -> &[i32] {
        match self.rows.get(&id) {
            _ if id == 0 => &self.root,
            Some(Row::Submenu { children, .. }) => children,
            _ => &[],
        }
    }

    /// `id`'s dbusmenu properties.
    fn properties(&self, id: i32) -> HashMap<String, OwnedValue> {
        let mut p = HashMap::new();
        let mut put = |k: &str, v: Value<'_>| {
            if let Ok(v) = v.try_to_owned() {
                p.insert(k.to_string(), v);
            }
        };
        match self.rows.get(&id) {
            None if id == 0 => put("children-display", Value::from("submenu")),
            None => {}
            Some(Row::Submenu { label, .. }) => {
                put("label", Value::from(mnemonic_free(label)));
                put("children-display", Value::from("submenu"));
            }
            Some(Row::Item { label, enabled, checked, .. }) => {
                put("label", Value::from(mnemonic_free(label)));
                put("enabled", Value::from(*enabled));
                if let Some(on) = checked {
                    put("toggle-type", Value::from("checkmark"));
                    put("toggle-state", Value::from(i32::from(*on)));
                }
            }
        }
        p
    }

    /// `id` and `depth` levels of its children (`-1`: all), the reply to `GetLayout`.
    fn layout_item(&self, id: i32, depth: i32) -> LayoutItem {
        let children = if depth == 0 {
            Vec::new()
        } else {
            self.children(id).iter().filter_map(|&c| self.layout(c, depth.saturating_sub(1)).try_to_owned().ok()).collect()
        };
        LayoutItem { id, properties: self.properties(id), children }
    }

    /// [`Shared::layout_item`] as a variant's value: how dbusmenu nests the children.
    fn layout(&self, id: i32, depth: i32) -> Value<'static> {
        let children: Vec<Value<'static>> =
            if depth == 0 { Vec::new() } else { self.children(id).iter().map(|&c| self.layout(c, depth.saturating_sub(1))).collect() };
        let children = zbus::zvariant::Array::from(children);
        let built = StructureBuilder::new().add_field(id).add_field(self.properties(id)).append_field(Value::Array(children)).build();
        built.map(Value::Structure).unwrap_or_else(|_| Value::from(0))
    }
}

/// A label with its underscores doubled: dbusmenu reads a single one as a mnemonic.
fn mnemonic_free(label: &str) -> String {
    label.replace('_', "__")
}

/// One item of the menu tree as dbusmenu sends it: `(ia{sv}av)`, each child a variant holding the
/// same structure.
#[derive(Debug, serde::Serialize, zbus::zvariant::Type)]
struct LayoutItem {
    id: i32,
    properties: HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
}

/// The `com.canonical.dbusmenu` object.
struct MenuObject {
    shared: Arc<Mutex<Shared>>,
    commands: Mutex<Sender<String>>,
    ctx: egui::Context,
}

impl MenuObject {
    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// An enabled item's command goes to the app's inbox; anything else (a hover, a submenu, a
    /// disabled item) does nothing.
    fn chosen(&self, id: i32, kind: &str) {
        if kind != "clicked" {
            return;
        }
        let command = match self.lock().rows.get(&id) {
            Some(Row::Item { id, enabled: true, .. }) => id.clone(),
            _ => return,
        };
        // The app gone means nothing is left to run it.
        let _ = self.commands.lock().unwrap_or_else(PoisonError::into_inner).send(command);
        self.ctx.request_repaint();
    }
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl MenuObject {
    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }
    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".into()
    }
    #[zbus(property)]
    fn status(&self) -> String {
        "normal".into()
    }
    #[zbus(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_layout(&self, parent_id: i32, recursion_depth: i32, _property_names: Vec<String>) -> (u32, LayoutItem) {
        let s = self.lock();
        (s.revision, s.layout_item(parent_id, recursion_depth))
    }

    fn get_group_properties(&self, ids: Vec<i32>, _property_names: Vec<String>) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let s = self.lock();
        // An empty list asks for every item (the dbusmenu contract).
        let ids = if ids.is_empty() { s.rows.keys().copied().collect() } else { ids };
        ids.into_iter().filter(|id| s.rows.contains_key(id)).map(|id| (id, s.properties(id))).collect()
    }

    fn get_property(&self, id: i32, name: String) -> OwnedValue {
        self.lock().properties(id).remove(&name).unwrap_or_else(|| OwnedValue::from(0i32))
    }

    fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
        self.chosen(id, &event_id);
    }

    fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
        for (id, kind, _, _) in events {
            self.chosen(id, &kind);
        }
        Vec::new()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let s = self.lock();
        let unknown = ids.into_iter().filter(|id| *id != 0 && !s.rows.contains_key(id)).collect();
        (Vec::new(), unknown)
    }
}

/// Whether the registrar on the bus is Nodalix's, the one registrar that finds a window's menu by
/// its process.
fn nodalix_registrar(conn: &Connection) -> bool {
    let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(conn) else { return false };
    let owner = |name: &str| zbus::names::BusName::try_from(name).ok().and_then(|n| dbus.get_name_owner(n).ok()).map(|o| o.to_string());
    matches!((owner(REGISTRAR), owner(NODALIX_MENU)), (Some(r), Some(n)) if r == n)
}

/// Hand the menu to the registrar, and record whether it took it.
fn register(conn: &Connection, shown: &AtomicBool, ctx: &egui::Context) {
    let path = zbus::zvariant::ObjectPath::from_static_str_unchecked(MENU_PATH);
    let accepted =
        nodalix_registrar(conn) && conn.call_method(Some(REGISTRAR), REGISTRAR_PATH, Some(REGISTRAR), "RegisterWindow", &(std::process::id(), path)).is_ok();
    shown.store(accepted, Ordering::Relaxed);
    ctx.request_repaint();
    log::info!("global menu: {}", if accepted { "shown in the desktop's top bar" } else { "kept in the window" });
}

/// What the app keeps of the global menu.
pub struct GlobalMenu {
    /// The chosen items' command ids, for [`TomaApp::command_inbox`].
    pub commands: Receiver<String>,
    /// Takes a changed menu tree to the desktop.
    pub update: Box<dyn FnMut(&[Item])>,
    /// Whether the desktop shows the menus now (the in-window bar hides while it does).
    pub shown: Box<dyn Fn() -> bool>,
}

/// Export the menus and register them with the desktop's registrar, now and whenever a registrar
/// appears. `None` (the menus stay in the window) without a session bus.
pub fn install(ctx: &egui::Context, app: &TomaApp) -> Option<GlobalMenu> {
    let (rows, root) = Shared::rows_of(&menu_items(app));
    let shared = Arc::new(Mutex::new(Shared { rows, root, revision: 1 }));
    let (tx, commands) = channel();
    let object = MenuObject { shared: shared.clone(), commands: Mutex::new(tx), ctx: ctx.clone() };
    let conn = match zbus::blocking::connection::Builder::session().and_then(|b| b.serve_at(MENU_PATH, object)).and_then(|b| b.build()) {
        Ok(conn) => conn,
        Err(e) => {
            log::info!("global menu: no session bus ({e}); the menus stay in the window");
            return None;
        }
    };
    let shown = Arc::new(AtomicBool::new(false));
    register(&conn, &shown, ctx);
    // The registrar may start after Toma, or restart with the shell: register whenever it
    // appears, and take the menus back into the window while it is gone.
    let (watch_conn, watch_shown, watch_ctx) = (conn.clone(), shown.clone(), ctx.clone());
    let spawned = std::thread::Builder::new().name("global-menu".into()).spawn(move || {
        let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(&watch_conn) else { return };
        let Ok(changes) = dbus.receive_name_owner_changed() else { return };
        for change in changes {
            let Ok(args) = change.args() else { continue };
            if args.name().as_str() != REGISTRAR && args.name().as_str() != NODALIX_MENU {
                continue;
            }
            if args.new_owner().is_some() {
                register(&watch_conn, &watch_shown, &watch_ctx);
            } else {
                watch_shown.store(false, Ordering::Relaxed);
                watch_ctx.request_repaint();
            }
        }
    });
    if let Err(e) = spawned {
        log::warn!("global menu: couldn't watch the registrar ({e})");
    }
    let update = Box::new(move |items: &[Item]| {
        let (rows, root) = Shared::rows_of(items);
        let revision = {
            let mut s = shared.lock().unwrap_or_else(PoisonError::into_inner);
            if s.rows == rows && s.root == root {
                return;
            }
            s.rows = rows;
            s.root = root;
            s.revision = s.revision.wrapping_add(1);
            s.revision
        };
        // The desktop reads the menus again when told they changed; a failure leaves it on the
        // last ones it read.
        let _ = conn.emit_signal(None::<&str>, MENU_PATH, MENU_INTERFACE, "LayoutUpdated", &(revision, 0i32));
    });
    Some(GlobalMenu { commands, update, shown: Box::new(move || shown.load(Ordering::Relaxed)) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, label: &str, path: &[&str], enabled: bool) -> Item {
        Item { id: id.into(), label: label.into(), path: path.iter().map(|s| s.to_string()).collect(), shortcut: None, enabled, checked: Some(false) }
    }

    fn items() -> Vec<Item> {
        vec![
            item("edit.undo", "Undo", &["Edit"], true),
            item("file.new_doc", "New_Doc", &["File"], true),
            item("file.openRecent.0", "a.fcproj", &["File", "Open Recent"], true),
            item("file.clearRecent", "Clear", &["File", "Open Recent"], false),
            item("file.save", "Save", &["File"], true),
            item("nowhere", "Lost", &["Nowhere"], true),
        ]
    }

    #[test]
    fn rows_follow_the_in_window_bar() {
        let (rows, root) = Shared::rows_of(&items());
        // File comes before Edit, as in the bar; a menu not in the bar is left out.
        assert_eq!(root, vec![1, 7]);
        let s = Shared { rows, root, ..Default::default() };
        let label = |id: i32| s.properties(id).get("label").and_then(|v| <&str>::try_from(v).ok().map(str::to_string));
        assert_eq!(label(1).as_deref(), Some("File"));
        // The submenu sits where its first item was; mnemonics are escaped; checks are shown.
        assert_eq!(s.children(1), &[2, 3, 6]);
        assert_eq!(label(2).as_deref(), Some("New__Doc"));
        assert_eq!(label(3).as_deref(), Some("Open Recent"));
        assert_eq!(s.children(3), &[4, 5]);
        assert_eq!(s.properties(2).get("toggle-type").and_then(|v| <&str>::try_from(v).ok()), Some("checkmark"));
        // The layout carries the whole tree, typed as dbusmenu reads it.
        use zbus::zvariant::Type as _;
        assert_eq!(LayoutItem::SIGNATURE.to_string(), "(ia{sv}av)");
        assert_eq!(s.layout_item(0, -1).children.len(), 2);
        assert!(s.layout_item(0, 0).children.is_empty());
    }

    #[test]
    fn a_click_sends_the_command_and_a_disabled_item_nothing() {
        let (rows, root) = Shared::rows_of(&items());
        let (tx, rx) = channel();
        let object =
            MenuObject { shared: Arc::new(Mutex::new(Shared { rows, root, ..Default::default() })), commands: Mutex::new(tx), ctx: egui::Context::default() };
        let id_of = |command: &str| object.lock().rows.iter().find_map(|(k, r)| matches!(r, Row::Item { id, .. } if id == command).then_some(*k)).unwrap_or(-1);
        let (save, clear) = (id_of("file.save"), id_of("file.clearRecent"));
        object.chosen(save, "clicked");
        object.chosen(clear, "clicked");
        object.chosen(save, "hovered");
        assert_eq!(rx.try_iter().collect::<Vec<_>>(), vec!["file.save".to_string()]);
    }
}
