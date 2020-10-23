#[allow(unused_macros)]
use std::{fmt::Debug, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct KeyboardEventArgs {
  pub shift_pressed: bool,
  pub control_pressed: bool,
  /// The semantic meaning of the key currently being pressed, if there is
  /// one.
  pub key: Option<VirtualKeyCode>,
}

#[allow(dead_code)]
impl KeyboardEventArgs {
  /// Create a new [`KeyboardEventArgs`] which just presses a key.
  pub fn pressing(key: VirtualKeyCode) -> Self {
    KeyboardEventArgs {
      key: Some(key),
      ..Default::default()
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum VirtualKeyCode {
  Escape,
  Enter,
  Control,
  Shift,
  Alt,
  Left,
  Up,
  Right,
  Down,
  Backspace,
  Return,
  Space,
  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
  I,
  J,
  K,
  L,
  M,
  N,
  O,
  P,
  Q,
  R,
  S,
  T,
  U,
  V,
  W,
  X,
  Y,
  Z,
  a,
  b,
  c,
  d,
  e,
  f,
  g,
  h,
  i,
  j,
  k,
  l,
  m,
  n,
  o,
  p,
  q,
  r,
  s,
  t,
  u,
  v,
  w,
  x,
  y,
  z,
  Key1,
  Key2,
  Key3,
  Key4,
  Key5,
  Key6,
  Key7,
  Key8,
  Key9,
  Key0,
}

impl FromStr for VirtualKeyCode {
  type Err = &'static str;

  /// Parses a `KeyboardEvent.key` based on the equivalent name provided in
  /// [the W3C spec][spec].
  ///
  /// [spec]: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "A" => Ok(VirtualKeyCode::A),
      "B" => Ok(VirtualKeyCode::B),
      "C" => Ok(VirtualKeyCode::C),
      "D" => Ok(VirtualKeyCode::D),
      "E" => Ok(VirtualKeyCode::E),
      "F" => Ok(VirtualKeyCode::F),
      "G" => Ok(VirtualKeyCode::G),
      "H" => Ok(VirtualKeyCode::H),
      "I" => Ok(VirtualKeyCode::I),
      "J" => Ok(VirtualKeyCode::J),
      "K" => Ok(VirtualKeyCode::K),
      "L" => Ok(VirtualKeyCode::L),
      "M" => Ok(VirtualKeyCode::M),
      "N" => Ok(VirtualKeyCode::N),
      "O" => Ok(VirtualKeyCode::O),
      "P" => Ok(VirtualKeyCode::P),
      "Q" => Ok(VirtualKeyCode::Q),
      "R" => Ok(VirtualKeyCode::R),
      "S" => Ok(VirtualKeyCode::S),
      "T" => Ok(VirtualKeyCode::T),
      "U" => Ok(VirtualKeyCode::U),
      "V" => Ok(VirtualKeyCode::V),
      "W" => Ok(VirtualKeyCode::W),
      "X" => Ok(VirtualKeyCode::X),
      "Y" => Ok(VirtualKeyCode::Y),
      "Z" => Ok(VirtualKeyCode::Z),
      "a" => Ok(VirtualKeyCode::a),
      "b" => Ok(VirtualKeyCode::b),
      "c" => Ok(VirtualKeyCode::c),
      "d" => Ok(VirtualKeyCode::d),
      "e" => Ok(VirtualKeyCode::e),
      "f" => Ok(VirtualKeyCode::f),
      "g" => Ok(VirtualKeyCode::g),
      "h" => Ok(VirtualKeyCode::h),
      "i" => Ok(VirtualKeyCode::i),
      "j" => Ok(VirtualKeyCode::j),
      "k" => Ok(VirtualKeyCode::k),
      "l" => Ok(VirtualKeyCode::l),
      "m" => Ok(VirtualKeyCode::m),
      "n" => Ok(VirtualKeyCode::n),
      "o" => Ok(VirtualKeyCode::o),
      "p" => Ok(VirtualKeyCode::p),
      "q" => Ok(VirtualKeyCode::q),
      "r" => Ok(VirtualKeyCode::r),
      "s" => Ok(VirtualKeyCode::s),
      "t" => Ok(VirtualKeyCode::t),
      "u" => Ok(VirtualKeyCode::u),
      "v" => Ok(VirtualKeyCode::v),
      "w" => Ok(VirtualKeyCode::w),
      "x" => Ok(VirtualKeyCode::x),
      "y" => Ok(VirtualKeyCode::y),
      "z" => Ok(VirtualKeyCode::z),
      "0" => Ok(VirtualKeyCode::Key0),
      "1" => Ok(VirtualKeyCode::Key1),
      "2" => Ok(VirtualKeyCode::Key2),
      "3" => Ok(VirtualKeyCode::Key3),
      "4" => Ok(VirtualKeyCode::Key4),
      "5" => Ok(VirtualKeyCode::Key5),
      "6" => Ok(VirtualKeyCode::Key6),
      "7" => Ok(VirtualKeyCode::Key7),
      "8" => Ok(VirtualKeyCode::Key8),
      "9" => Ok(VirtualKeyCode::Key9),
      "Enter" => Ok(VirtualKeyCode::Enter),
      "Backspace" => Ok(VirtualKeyCode::Backspace),
      "Escape" => Ok(VirtualKeyCode::Escape),
      "Shift" => Ok(VirtualKeyCode::Shift),
      "Control" => Ok(VirtualKeyCode::Control),
      "Alt" => Ok(VirtualKeyCode::Alt),
      _ => Err("Unknown KeyboardEvent key"),
    }
  }
}
