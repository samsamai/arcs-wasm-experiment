use crate::modes::{
  ApplicationContext, Idle, KeyboardEventArgs, MouseEventArgs, State, Transition, VirtualKeyCode,
};

use arcs::specs::prelude::*;
use arcs::{
  components::{AddLine, AddPoint, CursorPosition, Delete, DrawingObject, Geometry, Selected},
  euclid::Point2D,
  DrawingSpace,
};

#[derive(Debug)]
pub struct AddLineMode {
  nested: Box<dyn State>,
}

impl AddLineMode {
  fn handle_transition(&mut self, transition: Transition) {
    match transition {
      Transition::ChangeState(new_state) => {
        log::debug!("Changing state {:?} -> {:?}", self.nested, new_state);
        self.nested = new_state;
      }
      Transition::DoNothing => {}
    }
  }
}
impl State for AddLineMode {
  fn on_mouse_down(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    let trans = self.nested.on_mouse_down(ctx, args);
    self.handle_transition(trans);
    Transition::DoNothing
  }

  fn on_mouse_up(&mut self, ctx: &mut dyn ApplicationContext, args: &MouseEventArgs) -> Transition {
    let trans = self.nested.on_mouse_up(ctx, args);
    self.handle_transition(trans);
    Transition::DoNothing
  }

  fn on_key_pressed(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &KeyboardEventArgs,
  ) -> Transition {
    if args.key == Some(VirtualKeyCode::Escape) {
      // pressing escape should take us back to idle
      self.nested.on_cancelled(ctx);
      return Transition::ChangeState(Box::new(Idle::default()));
    }

    let trans = self.nested.on_key_pressed(ctx, args);
    self.handle_transition(trans);
    Transition::DoNothing
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    let trans = self.nested.on_mouse_move(ctx, args);
    self.handle_transition(trans);
    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    self.nested.on_cancelled(ctx);
    self.nested = Box::new(WaitingToPlaceStart::default());
  }
}

impl Default for AddLineMode {
  fn default() -> AddLineMode {
    AddLineMode {
      nested: Box::new(WaitingToPlaceStart::default()),
    }
  }
}

///////////////////////////////////////////////////////
/// Start Vertex
// The base sub-state for [`AddPointMode`]. We're waiting for the user to click
/// so we can start adding a point to the canvas.
#[derive(Debug, Default)]
struct WaitingToPlaceStart;

impl State for WaitingToPlaceStart {
  fn on_mouse_down(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("WaitingToPlace on_mouse_down called");

    // make sure nothing else is selected
    ctx.unselect_all();

    let layer = ctx.default_layer();

    let command_entity = ctx.command();
    let mut storage: WriteStorage<AddPoint> = ctx.world_mut().write_storage();
    let _ = storage.insert(
      command_entity,
      AddPoint {
        location: args.location,
        layer,
      },
    );

    Transition::ChangeState(Box::new(PlacingStart {}))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    ctx.suppress_redraw();
    let mut cursor_position = ctx.world_mut().write_resource::<CursorPosition>();
    cursor_position.location = args.location;

    Transition::DoNothing
  }
}

#[derive(Debug)]
struct PlacingStart;

impl State for PlacingStart {
  fn on_mouse_up(&mut self, ctx: &mut dyn ApplicationContext, args: &MouseEventArgs) -> Transition {
    log::debug!("PlacingStart on_mouse_up called");

    let command_entity = ctx.command();
    let layer = ctx.default_layer();
    let mut world = ctx.world_mut();

    let entities: Entities = world.entities();
    let selecteds: ReadStorage<Selected> = world.read_storage();
    let drawing_objects: ReadStorage<DrawingObject> = world.read_storage();

    let mut effective_location: Point2D<f64, DrawingSpace> = Point2D::new(0., 0.);
    for (entity, selected, drawing_object) in (&entities, &selecteds, &drawing_objects).join() {
      if let Geometry::Point(point) = drawing_object.geometry {
        effective_location = point;
        break;
      };
    }

    let mut storage: WriteStorage<AddLine> = world.write_storage();
    let _ = storage.insert(
      command_entity,
      AddLine {
        location: effective_location,
        layer,
      },
    );

    let mut storage: WriteStorage<Delete> = world.write_storage();
    let _ = storage.insert(command_entity, Delete {});

    Transition::ChangeState(Box::new(WaitingToPlaceEnd {}))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    let mut cursor_position = ctx.world_mut().write_resource::<CursorPosition>();
    cursor_position.location = args.location;

    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    // make sure we clean up the temporary point.
    let command_entity = ctx.command();
    let mut storage: WriteStorage<Delete> = ctx.world_mut().write_storage();
    let _ = storage.insert(command_entity, Delete {});
  }
}

///////////////////////////////////////////////////////
/// End Vertex
// The base sub-state for [`AddPointMode`]. We're waiting for the user to click
/// so we can start adding a point to the canvas.
#[derive(Debug)]
struct WaitingToPlaceEnd;

impl State for WaitingToPlaceEnd {
  fn on_mouse_down(
    &mut self,
    _ctx: &mut dyn ApplicationContext,
    _args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("WaitingToPlace on_mouse_down called");

    Transition::ChangeState(Box::new(PlacingEnd {}))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("PlacingEnd on_mouse_move called");

    let mut cursor_position = ctx.world_mut().write_resource::<CursorPosition>();
    cursor_position.location = args.location;

    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    // make sure we clean up the temporary line.
    let command_entity = ctx.command();
    let mut storage: WriteStorage<Delete> = ctx.world_mut().write_storage();
    let _ = storage.insert(command_entity, Delete {});
  }
}

#[derive(Debug)]
struct PlacingEnd;

impl State for PlacingEnd {
  fn on_mouse_up(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    _args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("PlacingEnd on_mouse_up called");
    ctx.unselect_all();

    Transition::ChangeState(Box::new(WaitingToPlaceStart::default()))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("PlacingEnd on_mouse_move called");

    let mut cursor_position = ctx.world_mut().write_resource::<CursorPosition>();
    cursor_position.location = args.location;

    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    // make sure we clean up the temporary line.
    let command_entity = ctx.command();
    let mut storage: WriteStorage<Delete> = ctx.world_mut().write_storage();
    let _ = storage.insert(command_entity, Delete {});
  }
}
