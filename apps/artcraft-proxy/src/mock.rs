//! Mock mode: canned assistant replies and procedurally generated placeholder
//! images so the egui app is fully usable (and CI can exercise the API) without
//! a real `ZAI_API_KEY`.

use anyhow::Result;
use std::io::Cursor;

/// A helpful canned reply for mock chat sessions.
pub fn chat_reply(user_msg: &str) -> String {
  let trimmed = user_msg.trim();
  let short: String = trimmed.chars().take(80).collect();
  format!(
    "(mock mode — set ZAI_API_KEY on artcraft-proxy for real GLM answers)\n\n\
     You said: “{short}”\n\n\
     Here is a ready-to-paste image prompt inspired by it:\n\
     “{short} — cinematic lighting, rich color grading, intricate detail, \
     shot on 85mm, dramatic rim light, 4k concept art, trending on artstation”\n\n\
     Open the Image studio and press Generate to craft it."
  )
}

/// Renders a deterministic gradient/pattern PNG derived from a hash of the
/// prompt — mock images are always the same for the same prompt.
pub fn image_png(prompt: &str, size: &str) -> Result<Vec<u8>> {
  let (mut w, mut h) = (1024u32, 1024u32);
  if let Some((a, b)) = size.to_ascii_lowercase().split_once('x') {
    if let (Ok(a), Ok(b)) = (a.trim().parse::<u32>(), b.trim().parse::<u32>()) {
      w = a.clamp(64, 2048);
      h = b.clamp(64, 2048);
    }
  }

  let seed = prompt.bytes().fold(0x811c9dc5u32, |acc, b| (acc ^ b as u32).wrapping_mul(0x01000193));
  let (r0, g0, b0) = hue(seed);
  let (r1, g1, b1) = hue(seed.rotate_left(13));

  let mut img = image::RgbImage::new(w, h);
  for (x, y, pixel) in img.enumerate_pixels_mut() {
    let fx = x as f32 / (w - 1).max(1) as f32;
    let fy = y as f32 / (h - 1).max(1) as f32;
    // Diagonal blend + soft wave so it reads as "generated art", not a plain gradient.
    let wave = ((fx * 6.283_185_5 * 2.0 + fy * 3.0).sin() + 1.0) * 0.5;
    let t = (fx * 0.6 + fy * 0.4 + wave * 0.15).clamp(0.0, 1.0);
    *pixel = image::Rgb([
      (r0 as f32 + (r1 as f32 - r0 as f32) * t) as u8,
      (g0 as f32 + (g1 as f32 - g0 as f32) * t) as u8,
      (b0 as f32 + (b1 as f32 - b0 as f32) * t) as u8,
    ]);
  }

  let mut out = Cursor::new(Vec::new());
  image::DynamicImage::ImageRgb8(img).write_to(&mut out, image::ImageFormat::Png)?;
  Ok(out.into_inner())
}

/// Map a 32-bit seed to a pleasant RGB triple (golden-angle hue walk).
fn hue(seed: u32) -> (f32, f32, f32) {
  let h = (seed % 360) as f32;
  let (r, g, b) = hsv_to_rgb(h, 0.62, 0.85);
  (r * 255.0, g * 255.0, b * 255.0)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
  let c = v * s;
  let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
  let m = v - c;
  let (r, g, b) = match h as u32 % 360 {
    0..=59 => (c, x, 0.0),
    60..=119 => (x, c, 0.0),
    120..=179 => (0.0, c, x),
    180..=239 => (0.0, x, c),
    240..=299 => (x, 0.0, c),
    _ => (c, 0.0, x),
  };
  (r + m, g + m, b + m)
}
