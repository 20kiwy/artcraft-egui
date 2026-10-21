#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! ArtCraft, rebuilt in pure egui: an AI creative studio desktop app with
//! **no TypeScript and no webview**. It talks to `artcraft-proxy` (the ZAI
//! plugin bridge) whose address is hardcoded in [`state::DEFAULT_PROXY_URL`].

mod app;
mod icons;
mod lightbox;
mod pages;
mod proxy;
mod shell;
mod state;
mod theme;
mod titlebar;
mod widgets;

const APP_ID: &str = "ai.artcraft.egui";

fn main() -> eframe::Result {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
  // 840x540 min: small laptop screens at high OS scaling still fit
  // (the in-app scale guard in app.rs keeps the logical viewport usable).
  let mut viewport = egui::ViewportBuilder::default()
    .with_title("ArtCraft")
    .with_app_id(APP_ID)
    .with_inner_size([1440.0, 900.0])
    .with_min_inner_size([840.0, 540.0]);
  // Test/dev hooks (no CLI needed): ARTCRAFT_WIN=1366x768 overrides the
  // initial window size, ARTCRAFT_PPP=1.5 forces a UI scale.
  if let Some((w, h)) = std::env::var("ARTCRAFT_WIN").ok().and_then(parse_size) {
    viewport = viewport.with_inner_size([w, h]).with_min_inner_size([360.0, 320.0]);
  }
  if let Some(icon) = app_icon() {
    viewport = viewport.with_icon(icon);
  }
  // The app draws its own brutalist title bar, like the upstream native client.
  viewport = viewport.with_decorations(false);
  let options = eframe::NativeOptions { viewport, persist_window: true, ..Default::default() };
  eframe::run_native("ArtCraft", options, Box::new(|cc| Ok(Box::new(app::ArtcraftApp::new(cc)))))
}

fn parse_size(s: String) -> Option<(f32, f32)> {
  let (w, h) = s.split_once(['x', 'X', '*'])?;
  Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

/// A generative "A" mark (accent triangle + ink crossbar) so we ship no
/// third-party logo assets — drawn at startup with the `image` crate.
fn app_icon() -> Option<egui::IconData> {
  const S: u32 = 64;
  let a = (10.0, 52.0); // triangle bottom-left
  let b = (32.0, 10.0); // apex
  let c = (54.0, 52.0); // triangle bottom-right
  let inside = |px: f32, py: f32| -> bool {
    let sign = |x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32| (x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1);
    let d1 = sign(px, py, a.0, a.1, b.0, b.1);
    let d2 = sign(px, py, b.0, b.1, c.0, c.1);
    let d3 = sign(px, py, c.0, c.1, a.0, a.1);
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
  };
  let mut img = image::RgbaImage::new(S, S);
  for (x, y, px) in img.enumerate_pixels_mut() {
    let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
    // Rounded dark tile.
    let corner = 12.0;
    let rx = fx.min(S as f32 - fx);
    let ry = fy.min(S as f32 - fy);
    let rounded = (rx * rx + ry * ry).sqrt() > corner || (rx >= corner && ry >= corner) || (rx >= corner || ry >= corner);
    let bg = if rounded { [0x12, 0x13, 0x16, 0xff] } else { [0x12, 0x13, 0x16, 0x00] };
    *px = if inside(fx, fy) { image::Rgba([0x2d, 0x81, 0xff, 0xff]) } else { image::Rgba(bg) };
  }
  // Ink crossbar across the triangle.
  for x in 22..42u32 {
    for y in 36..41u32 {
      img.put_pixel(x, y, image::Rgba([0xf2, 0xf1, 0xee, 0xff]));
    }
  }
  let rgba = img.into_raw();
  Some(egui::IconData { width: S, height: S, rgba })
}
