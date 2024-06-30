pub mod accordion;
pub mod aspect_ratio;
pub mod avatar;
pub mod checkbox;
pub mod collapsible;
pub(crate) mod dismissable_layer;
pub mod label;
pub mod primitive;
pub mod progress;
pub(crate) mod radio;
pub mod radio_group;
pub mod scroll_area;
pub mod separator;
pub mod slider;
pub mod slot;
pub mod switch;
pub mod tabs;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;

pub(crate) mod collection;
pub(crate) mod presence;
pub(crate) mod roving_focus;

#[derive(Default, Clone, PartialEq, Copy, strum_macros::Display)]
pub enum Direction {
  #[default]
  #[strum(to_string = "ltr")]
  LeftToRight,
  #[strum(to_string = "rtl")]
  RightToLeft,
}

#[derive(Default, Clone, PartialEq, Copy, strum_macros::Display)]
pub enum Orientation {
  #[default]
  #[strum(to_string = "horizontal")]
  Horizontal,
  #[strum(to_string = "vertical")]
  Vertical,
}
