//! Navigation model: pages, sidebar sections, layout constants and breadcrumbs
//! (mirrors the upstream shell: 40 px title bar, 232 px sidebar, 48 px header).

use crate::icons::Icon;

pub const SIDEBAR_WIDTH: f32 = 232.0;
pub const TITLEBAR_HEIGHT: f32 = 40.0;
pub const HEADER_HEIGHT: f32 = 48.0;
/// The content panel's gap to the window edges, and its corner radius.
pub const PANEL_GAP: i8 = 8;
pub const PANEL_RADIUS: u8 = 8;

/// Responsive floor: the smallest logical (scale-independent) viewport the
/// layout needs so the title bar, page header and the docked composer (text
/// area) always fit. On high-DPI/small displays the app eases its UI scale
/// down towards this floor instead of letting the composer fall off-window.
pub const MIN_LOGICAL_HEIGHT: f32 = 540.0;
pub const MIN_LOGICAL_WIDTH: f32 = 860.0;
/// Below this logical height the pages switch to compact composers
/// (2 rows instead of 3) to keep feed space comfortable.
pub const COMPACT_BELOW: f32 = 640.0;

/// Every page in the rebuild's navigation. The upstream remakes more tools
/// (video, audio, 3D…); this rebuild ships the ones the ZAI plugin serves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
  #[default]
  Home,
  CreateImage,
  Assistant,
  Library,
  Settings,
}

impl Page {
  pub fn label(self) -> &'static str {
    match self {
      Page::Home => "Home",
      Page::CreateImage => "Image",
      Page::Assistant => "Assistant",
      Page::Library => "Library",
      Page::Settings => "Settings",
    }
  }

  pub fn icon(self) -> Icon {
    match self {
      Page::Home => Icon::Home,
      Page::CreateImage => Icon::Image,
      Page::Assistant => Icon::Chat,
      Page::Library => Icon::Library,
      Page::Settings => Icon::Settings,
    }
  }

  /// The breadcrumb trail (`Create › Image`).
  pub fn crumbs(self) -> (&'static str, &'static str) {
    match self {
      Page::Home => ("ArtCraft", "Home"),
      Page::CreateImage | Page::Assistant => ("Create", self.label()),
      Page::Library => ("Assets", "Library"),
      Page::Settings => ("System", "Settings"),
    }
  }
}

pub const SECTIONS: [(&str, &[Page]); 3] = [
  ("Create", &[Page::CreateImage, Page::Assistant]),
  ("Assets", &[Page::Library]),
  ("System", &[Page::Settings]),
];
