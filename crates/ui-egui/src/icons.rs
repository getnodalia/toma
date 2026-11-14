//! Vector icons drawn in code on a 16×16 design grid (crisp at any DPI, recolourable, no
//! third-party or Adobe artwork). Shapes follow NLE conventions (ripple brackets, razor blade…).

use egui::epaint::{PathShape, PathStroke};
use egui::{Color32, Painter, Pos2, Rect, pos2, vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Icon {
    Selection,
    TrackSelectFwd,
    TrackSelectBack,
    Ripple,
    Rolling,
    RateStretch,
    Remix,
    Razor,
    Slip,
    Slide,
    Pen,
    Rectangle,
    Ellipse,
    Hand,
    Zoom,
    Type,
    Play,
    Pause,
    StepBack,
    StepFwd,
    GoToIn,
    GoToOut,
    /// Go to Previous / Next Edit Point: a triangle towards a cut (an I-beam: two clip edges meeting).
    PrevEdit,
    NextEdit,
    MarkIn,
    MarkOut,
    Marker,
    Insert,
    Overwrite,
    Lift,
    Extract,
    Camera,
    Loop,
    Wrench,
    Plus,
    Eye,
    EyeOff,
    Speaker,
    Mute,
    Lock,
    Unlock,
    SyncLock,
    Mic,
    Folder,
    Film,
    Sequence,
    Audio,
    Image,
    Search,
    ListView,
    IconView,
    Freeform,
    NewItem,
    Trash,
    Home,
    Workspaces,
    Hamburger,
    ChevronDown,
    ChevronRight,
    Magnet,
    Link,
    /// Open a web address: an arrow leaving an open frame.
    ExternalLink,
    Keyframe,
    Stopwatch,
    Fx,
    Reset,
    Close,
    Fullscreen,
    Export,
    /// An arrow down into a tray (Import).
    Import,
    Gear,
    Info,
    Captions,
    Adjust,
    Nest,
    Undo,
    Redo,
    Bell,
    /// Community chat (the Nodalia Discord): a speech bubble with three dots. Our own drawing,
    /// not the Discord logo.
    Chat,
    /// Website: a globe.
    Globe,
    /// Source code (GitHub repository): angle brackets and a slash.
    Code,
    Sparkle,
    Grid,
    Square,
    /// Toggle Proxies: a small frame inside a large one.
    Proxy,
    /// Offline media: a broken link.
    Offline,
    /// Mask tracking: backward continuously / one frame, forward one frame / continuously.
    TrackMaskBack,
    TrackMaskBackFrame,
    TrackMaskFwdFrame,
    TrackMaskFwd,
    /// Project panel footer: Sort Icons (lines of decreasing length).
    SortIcons,
    /// Project panel footer: Automate to Sequence (three clips in a row).
    Automate,
    /// Media Browser: a favourite (five-pointed star).
    Star,
    /// Media Browser: back / forward / up one level.
    ChevronLeft,
    ArrowUp,
    /// Media Browser: a local drive and a network location.
    Drive,
    Network,
    /// Media Browser: recent directories (a clock face).
    Clock,
    /// Appearance Mode: Auto (a display on a stand), Light (a sun), Dark (a crescent moon).
    Monitor,
    Sun,
    Moon,
    /// Colour parameters: pick a colour from the Program monitor (a pipette).
    Eyedropper,
}

pub struct Pen16<'a> {
    painter: &'a Painter,
    rect: Rect,
    color: Color32,
    width: f32,
}

impl Pen16<'_> {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        let s = self.rect.width().min(self.rect.height()) / 16.0;
        let o = self.rect.center() - vec2(8.0 * s, 8.0 * s);
        pos2(o.x + x * s, o.y + y * s)
    }
    fn s(&self) -> f32 {
        self.rect.width().min(self.rect.height()) / 16.0
    }
    /// Stroke width in points covering a whole number of device pixels, at least one (a half pixel
    /// rounds down: 1.25 pt at scale 1 is 1 px at 1x and 2 px at 2x), so lines never straddle pixels.
    fn stroke_w(&self) -> f32 {
        let ppp = self.painter.pixels_per_point();
        (self.width * self.s() * ppp - 0.5).ceil().max(1.0) / ppp
    }
    fn stroke(&self) -> PathStroke {
        PathStroke::new(self.stroke_w(), self.color)
    }
    /// Snap one coordinate (points) to the device pixel grid. `mid` puts it where the middle of a
    /// stroke must sit to cover whole pixels (a pixel centre for an odd pixel width, an edge for an
    /// even one); otherwise on a pixel edge (where a line ends or a fill begins). Offsets are rounded
    /// from the icon's centre, the same way on both sides, so equal steps on the 16 grid stay equal
    /// on screen (the three lines of the menu mark get the same two gaps).
    fn snap(&self, v: f32, centre: f32, mid: bool) -> f32 {
        let ppp = self.painter.pixels_per_point();
        let half = if mid && (self.stroke_w() * ppp).round() as i32 % 2 == 1 { 0.5 } else { 0.0 };
        let base = (centre * ppp).round() + half;
        (base + ((v - centre) * ppp).round()) / ppp
    }
    fn snap_pos(&self, q: Pos2, mid_x: bool, mid_y: bool) -> Pos2 {
        let c = self.rect.center();
        pos2(self.snap(q.x, c.x, mid_x), self.snap(q.y, c.y, mid_y))
    }
    /// Grid points to screen. Points on a horizontal or vertical segment (one that runs less than a
    /// device pixel sideways, so it draws straight anyway) snap to device pixels so the segment is
    /// drawn crisp; points between slanted segments keep their exact place.
    fn crisp(&self, pts: &[(f32, f32)], closed: bool) -> Vec<Pos2> {
        let n = pts.len();
        let ppp = self.painter.pixels_per_point();
        let q: Vec<Pos2> = pts.iter().map(|(x, y)| self.p(*x, *y)).collect();
        let seg = |i: usize, j: usize| {
            let d = (q[i] - q[j]) * ppp;
            (d.x.abs() < 1.0, d.y.abs() < 1.0) // (vertical, horizontal)
        };
        (0..n)
            .map(|i| {
                let mut nb = Vec::new();
                if i > 0 || closed {
                    nb.push(seg((i + n - 1) % n, i));
                }
                if i + 1 < n || closed {
                    nb.push(seg(i, (i + 1) % n));
                }
                let q = q[i];
                let vert = nb.iter().any(|s| s.0);
                let horiz = nb.iter().any(|s| s.1);
                if !vert && !horiz {
                    return q;
                }
                // across a straight segment: the stroke's middle; along it: a pixel edge, where it ends
                self.snap_pos(q, vert, horiz)
            })
            .collect()
    }
    fn line(&self, pts: &[(f32, f32)]) {
        let v = self.crisp(pts, false);
        self.painter.add(PathShape::line(v, self.stroke()));
    }
}

/// The NDL icon that draws `icon`: Toma draws every icon from Nodalia's bold set except the
/// panel menu mark, whose three lines it snaps to whole pixels itself.
fn ndl_name(icon: Icon) -> Option<&'static str> {
    use Icon::*;
    Some(match icon {
        Selection => "select",
        Pen => "pen",
        Rectangle => "rectangle",
        Ellipse => "ellipse",
        Hand => "pan",
        Zoom => "zoom",
        Type => "text",
        Play => "play",
        Pause => "pause",
        Camera => "camera",
        Loop => "repeat",
        Wrench | Gear => "settings",
        Plus => "plus",
        Eye => "view",
        EyeOff => "eye-off",
        Speaker => "volume",
        Mute => "mute",
        Lock => "lock",
        Unlock => "unlock",
        Mic => "microphone",
        Folder => "folder",
        Film => "film",
        Sequence => "timeline",
        Audio => "waveform",
        Image => "image",
        Search => "search",
        ListView => "list-view",
        IconView => "grid-view",
        NewItem => "new-item",
        Trash => "delete",
        Home => "home",
        Workspaces => "workspace",
        ChevronDown => "chevron-down",
        ChevronRight => "chevron-right",
        ChevronLeft => "chevron-left",
        Magnet => "magnet",
        Link => "link",
        Keyframe => "keyframe",
        Fx => "effects",
        Reset => "rotate-ccw",
        Close => "close",
        Fullscreen => "fullscreen",
        Export => "export",
        Import => "import",
        Info => "info",
        Captions => "subtitles",
        Undo => "undo",
        Redo => "redo",
        Bell => "bell",
        Chat => "comment",
        Globe => "globe",
        Code => "code",
        Sparkle => "sparkle",
        Grid => "grid",
        Offline => "offline",
        Star => "star",
        ArrowUp => "arrow-up",
        Clock | Stopwatch => "clock",
        Monitor => "monitor",
        Sun => "sun",
        Moon => "moon",
        Eyedropper => "eyedropper",
        Razor => "knife",
        Automate => "timeline",
        Adjust => "settings",
        Drive => "drive",
        ExternalLink => "external-link",
        Extract => "extract",
        Freeform => "freeform-view",
        GoToIn => "go-to-in",
        GoToOut => "go-to-out",
        Insert => "insert-edit",
        Lift => "lift",
        MarkIn => "mark-in",
        MarkOut => "mark-out",
        Marker => "marker",
        Nest => "nest",
        Network => "network",
        NextEdit => "next-edit",
        Overwrite => "overwrite-edit",
        PrevEdit => "previous-edit",
        Proxy => "proxy",
        RateStretch => "rate-stretch",
        Remix => "remix",
        Ripple => "ripple-edit",
        Rolling => "rolling-edit",
        Slide => "slide-edit",
        Slip => "slip",
        SortIcons => "sort",
        Square => "square",
        StepBack => "skip-previous",
        StepFwd => "skip-next",
        SyncLock => "sync-lock",
        TrackMaskBack => "rewind",
        TrackMaskBackFrame => "play-reverse",
        TrackMaskFwdFrame => "play",
        TrackMaskFwd => "fast-forward",
        TrackSelectBack => "track-select-backward",
        TrackSelectFwd => "track-select-forward",
        _ => return None,
    })
}

/// Paint `icon` centred in `rect`.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    if let Some(glyph) = ndl_name(icon).and_then(nodalia_theme::icons::get) {
        // NDL draws on a 24 grid with a 2 px margin; the menu mark fills a 16 grid edge to edge
        nodalia_theme::icons::paint(painter, rect.expand(rect.width() / 12.0), glyph, nodalia_theme::icons::Style::bold(color));
        return;
    }
    // the bold stroke of NDL's icons at this size (2.25 on 24 is 1.5 on 16)
    let pen = Pen16 { painter, rect, color, width: 1.5 };
    // only the panel menu mark is Toma's own: its three lines are snapped to whole pixels
    if icon == Icon::Hamburger {
        for y in [4.5, 8.0, 11.5] {
            pen.line(&[(3.0, y), (13.0, y)]);
        }
    }
}

/// An icon button: returns the response; `active` draws the selected state.
pub fn button(ui: &mut egui::Ui, icon: Icon, size: f32, active: bool, t: &crate::theme::Tokens, tooltip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), egui::Sense::click());
    let bg = if resp.is_pointer_button_down_on() {
        Some(t.pressed)
    } else if resp.hovered() {
        Some(t.hover)
    } else {
        None
    };
    if let Some(bg) = bg {
        ui.painter().rect_filled(rect, t.radius_sm, bg);
    }
    let col = if active {
        t.icon_active
    } else if resp.hovered() {
        t.tab_text_active
    } else {
        t.icon
    };
    paint(ui.painter(), rect.shrink(size * 0.2), icon, col);
    if tooltip.is_empty() { resp } else { resp.on_hover_text(tooltip) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Icon::*;
    use egui::epaint::{ColorMode, Shape};

    #[test]
    fn every_ndl_icon_toma_names_exists() {
        for &icon in ALL {
            if let Some(name) = ndl_name(icon) {
                assert!(nodalia_theme::icons::get(name).is_some(), "{icon:?} → {name}");
            }
        }
    }

    const ALL: &[Icon] = &[
        Monitor,
        Sun,
        Moon,
        Selection,
        TrackSelectFwd,
        TrackSelectBack,
        Ripple,
        Rolling,
        RateStretch,
        Remix,
        Razor,
        Slip,
        Slide,
        Pen,
        Rectangle,
        Ellipse,
        Hand,
        Zoom,
        Type,
        Play,
        Pause,
        StepBack,
        StepFwd,
        GoToIn,
        GoToOut,
        PrevEdit,
        NextEdit,
        MarkIn,
        MarkOut,
        Marker,
        Insert,
        Overwrite,
        Lift,
        Extract,
        Camera,
        Loop,
        Wrench,
        Plus,
        Eye,
        EyeOff,
        Speaker,
        Mute,
        Lock,
        Unlock,
        SyncLock,
        Mic,
        Folder,
        Film,
        Sequence,
        Audio,
        Image,
        Search,
        ListView,
        IconView,
        Freeform,
        NewItem,
        Trash,
        Home,
        Workspaces,
        Hamburger,
        ChevronDown,
        ChevronRight,
        Magnet,
        Link,
        ExternalLink,
        Keyframe,
        Stopwatch,
        Fx,
        Reset,
        Close,
        Fullscreen,
        Export,
        Import,
        Gear,
        Info,
        Captions,
        Adjust,
        Nest,
        Undo,
        Redo,
        Bell,
        Chat,
        Globe,
        Code,
        Sparkle,
        Grid,
        Square,
        Proxy,
        Offline,
        TrackMaskBack,
        TrackMaskBackFrame,
        TrackMaskFwdFrame,
        TrackMaskFwd,
        SortIcons,
        Automate,
        Star,
        ChevronLeft,
        ArrowUp,
        Drive,
        Network,
        Clock,
        Eyedropper,
    ];

    /// Paint `icon` at `ppp` device pixels per point into a rect of `size` points whose corner is
    /// off the pixel grid, and return what was drawn.
    fn drawn(icon: Icon, ppp: f32, size: f32) -> Vec<Shape> {
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(ppp);
        let mut shapes = Vec::new();
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                paint(ui.painter(), Rect::from_min_size(pos2(10.3, 7.7), vec2(size, size)), icon, Color32::WHITE);
            });
            out.textures_delta.clear();
            shapes = out.shapes.into_iter().map(|c| c.shape).collect();
        }
        fn flat(s: Shape, out: &mut Vec<Shape>) {
            match s {
                Shape::Vec(v) => v.into_iter().for_each(|s| flat(s, out)),
                s => out.push(s),
            }
        }
        let mut out = Vec::new();
        shapes.into_iter().for_each(|s| flat(s, &mut out));
        out
    }

    fn on_grid(v: f32, ppp: f32, width_px: f32) -> bool {
        let want = if width_px.round() as i32 % 2 == 1 { 0.5 } else { 0.0 };
        ((v * ppp).fract().abs() - want).abs() < 1e-3 || ((v * ppp).fract().abs() - (1.0 - want)).abs() < 1e-3
    }

    /// Every straight line of every icon covers whole device pixels at 1x and 2x (no half-pixel
    /// blur): a whole number of pixels wide, its middle where that width needs it.
    #[test]
    fn straight_lines_land_on_whole_pixels() {
        for &ppp in &[1.0, 2.0] {
            for &size in &[16.0, 13.0, 9.6, 20.0] {
                for &icon in ALL.iter().filter(|&&i| ndl_name(i).is_none()) {
                    for shape in drawn(icon, ppp, size) {
                        if let Shape::Path(p) = shape {
                            if p.stroke.width == 0.0 || matches!(p.stroke.color, ColorMode::Solid(c) if c == Color32::TRANSPARENT) {
                                continue;
                            }
                            let w = p.stroke.width * ppp;
                            assert!((w - w.round()).abs() < 1e-3 && w >= 1.0, "{icon:?} at {ppp}x size {size}: width {w} px");
                            let n = p.points.len();
                            let segs = if p.closed { n } else { n - 1 };
                            for i in 0..segs {
                                let (a, b) = (p.points[i], p.points[(i + 1) % n]);
                                if (a.y - b.y).abs() < 1e-4 {
                                    assert!(on_grid(a.y, ppp, w), "{icon:?} at {ppp}x size {size}: horizontal line at y {} px", a.y * ppp);
                                }
                                if (a.x - b.x).abs() < 1e-4 {
                                    assert!(on_grid(a.x, ppp, w), "{icon:?} at {ppp}x size {size}: vertical line at x {} px", a.x * ppp);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The panel menu mark (≡): three lines, the same two gaps, at every scale.
    #[test]
    fn menu_mark_lines_are_evenly_spaced() {
        for &ppp in &[1.0, 1.5, 2.0] {
            for &size in &[16.0, 13.0, 9.6, 12.0] {
                let ys: Vec<f32> = drawn(Icon::Hamburger, ppp, size)
                    .into_iter()
                    .filter_map(|s| if let Shape::Path(p) = s { Some(p.points[0].y * ppp) } else { None })
                    .collect();
                assert_eq!(ys.len(), 3);
                let (g1, g2) = (ys[1] - ys[0], ys[2] - ys[1]);
                assert!((g1 - g2).abs() < 1e-3 && g1 >= 2.0, "{ppp}x size {size}: gaps {g1} and {g2} px");
            }
        }
    }
}
