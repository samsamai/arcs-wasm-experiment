use super::keyboard_event_args::{KeyboardEventArgs, VirtualKeyCode};
use arcs::{euclid::Point2D, CanvasSpace};
use web_sys::KeyboardEvent;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Msg {
  Rendered,
  MouseDown(Point2D<f64, CanvasSpace>),
  MouseUp(Point2D<f64, CanvasSpace>),
  MouseMove(Point2D<f64, CanvasSpace>),
  KeyPressed(KeyboardEventArgs),
  WindowResized,
}

impl Msg {
  pub fn from_key_press(ev: KeyboardEvent) -> Self {
    let key = match ev.key().parse::<VirtualKeyCode>() {
      Ok(got) => Some(got),
      Err(_) => {
        // encountered an unknown key code, log it so we can update the
        // FromStr impl
        log::warn!("Encountered an unknown key: {}", ev.key());
        None
      }
    };

    Msg::KeyPressed(KeyboardEventArgs {
      shift_pressed: ev.shift_key(),
      control_pressed: ev.ctrl_key(),
      key,
    })
  }
}
