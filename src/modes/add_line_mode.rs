use crate::modes::{
  AddArcMode, AddPointMode, ApplicationContext, Idle, KeyboardEventArgs, MouseEventArgs, State,
  Transition, VirtualKeyCode,
};

use crate::msg::ButtonType;
use arcs::components::{DrawingObject, Geometry, Selected};
use arcs::specs::prelude::*;

#[derive(Debug, Default)]
pub struct AddLineMode;

impl AddLineMode {
  // fn handle_transition(&mut self, transition: Transition) {
  //   match transition {
  //     Transition::ChangeState(new_state) => {
  //       log::debug!("Changing state {:?} -> {:?}", self.nested, new_state);
  //       self.nested = new_state;
  //     }
  //     Transition::DoNothing => {}
  //   }
  // }
}
impl State for AddLineMode {
  // fn on_mouse_down(
  //   &mut self,
  //   ctx: &mut dyn ApplicationContext,
  //   args: &MouseEventArgs,
  // ) -> Transition {
  //   let trans = self.nested.on_mouse_down(ctx, args);
  //   self.handle_transition(trans);
  //   Transition::DoNothing
  // }

  // fn on_mouse_up(&mut self, ctx: &mut dyn ApplicationContext, args: &MouseEventArgs) -> Transition {
  //   let trans = self.nested.on_mouse_up(ctx, args);
  //   self.handle_transition(trans);
  //   Transition::DoNothing
  // }

  fn on_key_pressed(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &KeyboardEventArgs,
  ) -> Transition {
    if args.key == Some(VirtualKeyCode::Escape) {
      // pressing escape should take us back to idle
      // self.nested.on_cancelled(ctx);
      return Transition::ChangeState(Box::new(Idle::default()));
    }

    // let trans = self.nested.on_key_pressed(ctx, args);
    // self.handle_transition(trans);
    Transition::DoNothing
  }

  // fn on_mouse_move(
  //   &mut self,
  //   ctx: &mut dyn ApplicationContext,
  //   args: &MouseEventArgs,
  // ) -> Transition {
  //   let trans = self.nested.on_mouse_move(ctx, args);
  //   self.handle_transition(trans);
  //   Transition::DoNothing
  // }

  // fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
  //   self.nested.on_cancelled(ctx);
  //   self.nested = Box::new(WaitingToPlace::default());
  // }

  fn on_button_clicked(
    &mut self,
    _ctx: &mut dyn ApplicationContext,
    event_args: &ButtonType,
  ) -> Transition {
    match event_args {
      ButtonType::Arc => Transition::ChangeState(Box::new(AddArcMode::default())),
      ButtonType::Point => Transition::ChangeState(Box::new(AddPointMode::default())),
      ButtonType::Line => Transition::ChangeState(Box::new(AddLineMode::default())),
    }
  }
}
