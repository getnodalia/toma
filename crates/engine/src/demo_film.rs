//! The sample film: a real edit of Georges Méliès' *Le Voyage dans la Lune* (1902, public domain)
//! with a title card, chapter lower thirds, grades, punch-ins, dissolves, a music bed and markers.
//!
//! The footage is one small H.264 file of twelve scenes cut from the restored film (see
//! `assets/demo/voyage-dans-la-lune-scenes.mp4.attribution`), compiled into the binary and opened
//! from memory. The music is Offenbach's *Galop infernal* (1858, public domain), also embedded.
//! Nothing here touches the network.

use std::sync::Arc;

use toma_geom::Vec2;
use toma_project::graphic::{self, ensure_layer_uids, new_shape_layer, new_text_layer};
use toma_project::{
    BinId, ClipId, EffectInstance, ItemId, ItemKind, Label, MarkerId, MarkerKind, MediaClip, MediaRef, ParamValue, Project, SequenceSettings, TrackItem,
    TrackKind, Transition, TransitionId, find_effect, resolve_auto_points,
};
use toma_time::{FrameRate, TICKS_PER_SECOND, Tick, TimeRange};

use crate::MediaPool;

/// Twelve scenes cut from the 1902 film, 512x384, 24 fps, 109 s.
static FILM: &[u8] = include_bytes!("../../../assets/demo/voyage-dans-la-lune-scenes.mp4");
/// Offenbach, *Galop infernal* (Orpheus in the Underworld, 1858), 2:28.
static MUSIC: &[u8] = include_bytes!("../../../assets/demo/silent-cinema-galop.ogg");

const FILM_NAME: &str = "Le Voyage dans la Lune (1902) - scenes.mp4";
const MUSIC_NAME: &str = "Silent-cinema galop.ogg";
const FRAME: (u32, u32) = (1440, 1080);
const RATE: FrameRate = FrameRate::FPS_24;

fn secs(s: f64) -> Tick {
    RATE.snap(Tick::from_seconds_f64(s))
}

fn set(params: &mut std::collections::BTreeMap<String, toma_project::Param>, name: &str, value: ParamValue) {
    if let Some(p) = params.get_mut(name) {
        p.value = value;
    }
}

/// Write the embedded file beside the other temporary data so a saved project can reopen it; the
/// bare name stands in when the folder is not writable (and on the web, which has no folders).
fn materialise(name: &str, bytes: &[u8]) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = std::env::temp_dir().join("toma-sample-film");
        let path = dir.join(name);
        let current = std::fs::metadata(&path).map(|m| m.len() == bytes.len() as u64).unwrap_or(false);
        if current || (std::fs::create_dir_all(&dir).is_ok() && std::fs::write(&path, bytes).is_ok()) {
            return path.to_string_lossy().into_owned();
        }
    }
    name.to_string()
}

fn add_embedded(p: &mut Project, pool: &MediaPool, name: &str, bytes: &'static [u8], label: Label, bin: BinId) -> Option<ItemId> {
    let source = {
        let openers = pool.openers.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        toma_media::open_bytes(name, Arc::from(bytes), &openers).ok()?
    };
    let mut info = source.info().clone();
    info.name = name.to_string();
    let clip = MediaClip {
        media: MediaRef::File { path: materialise(name, bytes) },
        info,
        interpret: Default::default(),
        mark_in: None,
        mark_out: None,
        markers: vec![],
        offline: false,
        proxy: None,
        identity: None,
    };
    let key = crate::media_pool::media_key(&clip);
    let id = p.add_item(name, label, ItemKind::Media(Box::new(clip)), Some(bin));
    pool.insert_keyed(id, key, source);
    Some(id)
}

/// A graphic clip with `layers` at `start`, shown for `dur` seconds.
fn graphic_clip(p: &mut Project, name: &str, start: f64, dur: f64, layers: Vec<EffectInstance>) -> Option<TrackItem> {
    let src = crate::graphics::graphic_source(p, FRAME.0, FRAME.1, RATE);
    let mut ti = p.make_track_item(src, TrackKind::Video, secs(start), TimeRange::new(Tick::ZERO, secs(dur)), RATE)?;
    ti.name = name.to_string();
    ti.label = Label::Rose;
    ti.effects.extend(layers);
    ensure_layer_uids(&mut ti.effects);
    for e in &mut ti.effects {
        resolve_auto_points(e, FRAME, FRAME);
    }
    Some(ti)
}

fn text(t: &str, x: f64, y: f64, size: f64, style: &str, align: u32, tracking: f64) -> EffectInstance {
    let mut e = new_text_layer(t, Vec2::new(x, y), size);
    set(&mut e.params, "font", ParamValue::Text("Inter".into()));
    set(&mut e.params, "font_style", ParamValue::Text(style.into()));
    set(&mut e.params, "align", ParamValue::Choice(align));
    set(&mut e.params, "tracking", ParamValue::Float(tracking));
    set(&mut e.params, "fill_color", ParamValue::Color([0.99, 0.95, 0.84, 1.0]));
    set(&mut e.params, "shadow", ParamValue::Bool(true));
    set(&mut e.params, "shadow_opacity", ParamValue::Float(85.0));
    set(&mut e.params, "shadow_distance", ParamValue::Float(6.0));
    set(&mut e.params, "shadow_blur", ParamValue::Float(22.0));
    e
}

fn rule(x: f64, y: f64, w: f64, h: f64, color: [f32; 4]) -> EffectInstance {
    let mut e = new_shape_layer(0, Vec2::new(x, y), Vec2::new(w, h), Vec::new());
    set(&mut e.params, "fill_color", ParamValue::Color(color));
    set(&mut e.params, "stroke", ParamValue::Bool(false));
    e
}

/// Fade a clip in and out through its opacity (media time).
fn fade(ti: &mut TrackItem, inn: u32, out: u32) {
    let dur_frames = (ti.duration.seconds() * 24.0).round() as u32;
    if let Some(o) = ti.effect_mut("opacity").and_then(|o| o.params.get_mut("opacity")) {
        o.toggle_animation(Tick::ZERO);
        o.set_at(Tick::ZERO, ParamValue::Float(0.0));
        o.set_at(RATE.tick_of(inn as i64), ParamValue::Float(100.0));
        o.set_at(RATE.tick_of(dur_frames.saturating_sub(out) as i64), ParamValue::Float(100.0));
        o.set_at(RATE.tick_of(dur_frames.saturating_sub(1) as i64), ParamValue::Float(0.0));
    }
}

struct Cut {
    /// Seconds into the embedded file.
    src: f64,
    dur: f64,
    label: Label,
    /// Motion scale in percent and the position it pushes to (centre = 720,540).
    punch: Option<(f64, (f64, f64))>,
    /// Lumetri: temperature, contrast, saturation, vignette.
    grade: Option<(f64, f64, f64, f64)>,
}

const fn cut(src: f64, dur: f64, label: Label) -> Cut {
    Cut { src, dur, label, punch: None, grade: None }
}

fn cuts() -> Vec<Cut> {
    vec![
        // 1 The Astronomers' Club
        Cut { grade: Some((-6.0, 14.0, 105.0, -1.1)), ..cut(0.0, 6.0, Label::Mango) },
        // 2 The capsule is built: wide, then a punch-in on the work
        cut(8.0, 5.0, Label::Caribbean),
        Cut { punch: Some((135.0, (560.0, 560.0))), ..cut(13.0, 3.0, Label::Caribbean) },
        // 3 The foundry
        Cut { grade: Some((22.0, 18.0, 110.0, -0.9)), ..cut(16.0, 6.0, Label::Rose) },
        // 4 The cannon: wide, the gun, the crowd
        cut(24.0, 6.0, Label::Iris),
        Cut { punch: Some((165.0, (940.0, 700.0))), ..cut(30.0, 4.0, Label::Iris) },
        cut(34.0, 4.0, Label::Iris),
        // 5 The man in the Moon
        Cut { grade: Some((-18.0, 20.0, 100.0, -1.6)), ..cut(38.0, 6.0, Label::Cerulean) },
        Cut { grade: Some((-18.0, 20.0, 100.0, -1.6)), ..cut(44.0, 8.0, Label::Cerulean) },
        // 6 Landing and the lunar night
        cut(55.0, 6.0, Label::Forest),
        Cut { punch: Some((120.0, (780.0, 500.0))), ..cut(61.0, 6.0, Label::Lavender) },
        // 7 Mushroom cavern and the Selenite palace
        cut(69.0, 6.0, Label::Magenta),
        cut(77.0, 6.0, Label::Mango),
        // 8 The fall and the sea
        cut(85.0, 5.0, Label::Teal),
        Cut { grade: Some((-10.0, 12.0, 112.0, -0.8)), ..cut(93.0, 6.0, Label::Teal) },
        // 9 Parade
        cut(101.0, 7.0, Label::Yellow),
    ]
}

/// Build the sample film. `None` only when the embedded media cannot be opened.
pub fn film_project(pool: &MediaPool) -> Option<(Project, ItemId)> {
    let mut p = Project::new("A Trip to the Moon - Sample Film");
    let footage = p.add_bin("Footage", None);
    let audio = p.add_bin("Music", None);
    let titles = p.add_bin("Titles", None);
    let seqs = p.add_bin("Sequences", None);
    let _ = titles;

    let film = add_embedded(&mut p, pool, FILM_NAME, FILM, Label::Mango, footage)?;
    let music = add_embedded(&mut p, pool, MUSIC_NAME, MUSIC, Label::Caribbean, audio)?;

    let settings = SequenceSettings { width: FRAME.0, height: FRAME.1, frame_rate: RATE, preset: "HD 1440x1080 24".into(), ..SequenceSettings::default() };
    let seq = p.new_sequence("A Trip to the Moon", settings, 3, 3, Some(seqs));

    // V1: the picture edit
    let mut placed: Vec<(ClipId, Tick, Tick)> = Vec::new();
    let mut t = 0.0_f64;
    for c in cuts() {
        let mut v = p.make_track_item(film, TrackKind::Video, secs(t), TimeRange::new(secs(c.src), secs(c.dur)), RATE)?;
        v.label = c.label;
        v.scale_to_frame = true;
        for e in &mut v.effects {
            resolve_auto_points(e, FRAME, (512, 384));
        }
        if let Some((scale, (x, y))) = c.punch
            && let Some(m) = v.effect_mut("motion")
        {
            set(&mut m.params, "scale", ParamValue::Float(scale));
            set(&mut m.params, "position", ParamValue::Vec2(Vec2::new(x, y)));
        }
        if let Some((temp, contrast, sat, vig)) = c.grade
            && let Some(mut lum) = find_effect("lumetri").map(|d| d.instance())
        {
            set(&mut lum.params, "temperature", ParamValue::Float(temp));
            set(&mut lum.params, "contrast", ParamValue::Float(contrast));
            set(&mut lum.params, "saturation", ParamValue::Float(sat));
            set(&mut lum.params, "vignette_amount", ParamValue::Float(vig));
            v.effects.insert(0, lum);
        }
        placed.push((v.id, v.start, v.duration));
        p.sequence_mut(seq)?.video_tracks[0].items.push(v);
        t += c.dur;
    }
    let total = t;

    // V2: chapter lower thirds. V3: the title card and the closing card.
    let chapters: [(f64, &str, &str); 6] = [
        (6.5, "The capsule is built", "Chapter II"),
        (24.0, "The cannon", "Chapter IV"),
        (44.0, "The man in the Moon", "Chapter V"),
        (60.5, "The Selenites", "Chapter VII"),
        (77.0, "Splashdown", "Chapter VIII"),
        (66.0, "Mushroom cavern", "Chapter VI"),
    ];
    for (at, title, kicker) in chapters {
        let layers = vec![
            rule(214.0, 902.0, 6.0, 112.0, [0.93, 0.62, 0.22, 1.0]),
            text(kicker, 240.0, 876.0, 30.0, "Regular", 0, 6.0),
            text(title, 240.0, 936.0, 62.0, "Bold", 0, 0.0),
        ];
        let mut g = graphic_clip(&mut p, title, at, 4.0, layers)?;
        fade(&mut g, 8, 8);
        let s = p.sequence_mut(seq)?;
        s.video_tracks[1].items.push(g);
    }
    let title_layers = vec![
        rule(720.0, 540.0, 1440.0, 1080.0, [0.0, 0.0, 0.0, 0.42]),
        text("A TRIP TO THE MOON", 720.0, 470.0, 104.0, "Bold", 1, 4.0),
        text("Le Voyage dans la Lune", 720.0, 598.0, 46.0, "Regular", 1, 2.0),
        rule(720.0, 668.0, 360.0, 4.0, [0.93, 0.62, 0.22, 1.0]),
        text("A film by Georges Méliès  ·  1902", 720.0, 722.0, 32.0, "Regular", 1, 5.0),
    ];
    let mut title = graphic_clip(&mut p, "Title card", 0.5, 5.0, title_layers)?;
    title.label = Label::Mango;
    fade(&mut title, 14, 14);
    let end_layers = vec![
        rule(720.0, 540.0, 1440.0, 1080.0, [0.0, 0.0, 0.0, 0.5]),
        text("FIN", 720.0, 480.0, 170.0, "Bold", 1, 10.0),
        text("Restored hand-coloured print  ·  Public domain", 720.0, 650.0, 30.0, "Regular", 1, 3.0),
        text("Music: Offenbach, Galop infernal (1858)", 720.0, 700.0, 30.0, "Regular", 1, 3.0),
    ];
    let mut end = graphic_clip(&mut p, "End card", total - 5.0, 5.0, end_layers)?;
    end.label = Label::Mango;
    fade(&mut end, 18, 20);
    {
        let s = p.sequence_mut(seq)?;
        s.video_tracks[2].items.push(title);
        s.video_tracks[2].items.push(end);
    }

    // Transitions: a dissolve at the start of each new scene, dips between the big chapters.
    let tr_len = RATE.tick_of(18);
    let mk = |n: u64, eff: &str, from: Option<ClipId>, to: Option<ClipId>, start: Tick| Transition {
        id: TransitionId(900_000 + n),
        effect: crate::presets::effect_instance(eff),
        start,
        duration: tr_len,
        from,
        to,
        align: Default::default(),
        reverse: false,
    };
    let half = tr_len.mul_ratio(1, 2);
    // clip indices where the scene changes (and so a transition goes in)
    let scene_starts: [(usize, &str); 9] = [
        (1, "cross_dissolve"),
        (3, "dip_to_black"),
        (4, "cross_dissolve"),
        (7, "cross_dissolve"),
        (9, "dip_to_black"),
        (11, "cross_dissolve"),
        (13, "cross_dissolve"),
        (14, "cross_dissolve"),
        (15, "dip_to_black"),
    ];
    let mut trs = vec![mk(1, "dip_to_black", None, Some(placed.first()?.0), Tick::ZERO)];
    for (n, (i, eff)) in scene_starts.iter().enumerate() {
        let (to, start, _) = *placed.get(*i)?;
        let (from, _, _) = *placed.get(i - 1)?;
        trs.push(mk(2 + n as u64, eff, Some(from), Some(to), start - half));
    }
    p.sequence_mut(seq)?.video_tracks[0].transitions = trs;

    // A1: the music, trimmed to the picture, faded out; A2/A3 are named and left free for cuts.
    let mut m = p.make_track_item(music, TrackKind::Audio, Tick::ZERO, TimeRange::new(Tick::ZERO, secs(total)), RATE)?;
    m.label = Label::Caribbean;
    if let Some(l) = m.effect_mut("volume").and_then(|e| e.params.get_mut("level")) {
        l.toggle_animation(Tick::ZERO);
        l.set_at(Tick::ZERO, ParamValue::Float(-60.0));
        l.set_at(secs(3.0), ParamValue::Float(-9.0));
        l.set_at(secs(total - 5.0), ParamValue::Float(-9.0));
        l.set_at(secs(total - 0.1), ParamValue::Float(-60.0));
    }
    {
        let s = p.sequence_mut(seq)?;
        s.audio_tracks[0].items.push(m);
        for (tr, name) in s.video_tracks.iter_mut().zip(["Picture", "Chapters", "Titles"]) {
            tr.name = name.into();
        }
        for (tr, name) in s.audio_tracks.iter_mut().zip(["Music", "Effects", "Voice"]) {
            tr.name = name.into();
        }
        for (i, (sec, name, color)) in [
            (0.0, "Open: the Astronomers' Club", Label::Mango),
            (8.0, "Building the capsule", Label::Caribbean),
            (24.0, "The cannon", Label::Iris),
            (48.0, "Eye of the Moon", Label::Rose),
            (69.0, "Lunar caverns", Label::Magenta),
            (total - 5.0, "FIN", Label::Cerulean),
        ]
        .iter()
        .enumerate()
        {
            s.markers.push(toma_project::Marker {
                url: String::new(),
                author: String::new(),
                custom_color: None,
                id: MarkerId(800_000 + i as u64),
                start: secs(*sec),
                duration: Tick::ZERO,
                name: name.to_string(),
                comment: String::new(),
                kind: MarkerKind::Comment,
                color: *color,
            });
        }
        for tr in s.all_tracks_mut() {
            tr.sort();
        }
    }
    p.next_id = p.next_id.max(p.max_id().saturating_add(1));
    let _ = (TICKS_PER_SECOND, graphic::TEXT_LAYER);
    Some((p, seq))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sample_film_builds_and_is_consistent() {
        let pool = MediaPool::default();
        let (p, seq) = film_project(&pool).expect("the embedded media opens");
        let s = p.sequence(seq).expect("sequence");
        s.check().expect("a valid sequence");
        assert_eq!(s.video_tracks[0].items.len(), cuts().len());
        assert!(s.video_tracks[1].items.len() >= 5 && s.video_tracks[2].items.len() == 2);
        assert_eq!(s.audio_tracks[0].items.len(), 1);
    }

    #[test]
    fn the_command_opens_the_film() {
        let mut s = crate::Session::default();
        let r = s.execute("file.openFilmProject", serde_json::json!({})).expect("opens");
        assert!(r.get("sequence").is_some());
        let seq = s.state.active_sequence.expect("active sequence");
        let before = s.project.sequence(seq).map(|q| q.video_tracks[0].items.len()).unwrap_or(0);
        assert!(before >= 16);
        s.execute("timeline.select", serde_json::json!({"clips": []})).expect("a command still runs");
    }

    #[test]
    #[ignore = "writes frames for a human to look at"]
    fn render_preview_frames() {
        let pool = Arc::new(MediaPool::default());
        let (p, seq) = film_project(&pool).expect("film");
        let provider = pool.full_res_provider(Arc::new(p.clone()), Arc::new(crate::FsServices));
        let dir = std::env::var("FILM_FRAMES").unwrap_or_else(|_| "/tmp/film-frames".into());
        std::fs::create_dir_all(&dir).unwrap();
        for sec in [1.0, 4.0, 9.0, 12.0, 20.0, 27.0, 33.0, 49.0, 52.0, 66.0, 75.0, 83.0, 96.0] {
            let img = toma_render::render_sequence(&p, seq, secs(sec), toma_render::RenderOptions { captions: true, ..Default::default() }, &provider);
            let rgba = img.over_black_rgba8();
            let (w, h) = (img.w as u32, img.h as u32);
            image::RgbaImage::from_raw(w, h, rgba).unwrap().save(format!("{dir}/f{:05.1}.png", sec)).unwrap();
        }
    }
}
