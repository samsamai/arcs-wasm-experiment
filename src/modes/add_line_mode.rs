use crate::modes::{
  ApplicationContext, Idle, KeyboardEventArgs, MouseEventArgs, State, Transition, VirtualKeyCode,
};

use arcs::components::{Dimension, DrawingObject, Geometry, PointStyle, Selected};
use arcs::primitives::Line;
use arcs::specs::prelude::*;

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

    // create a point and automatically mark it as selected
    let temp_point = ctx
      .world_mut()
      .create_entity()
      .with(DrawingObject {
        geometry: Geometry::Point(args.location),
        layer,
      })
      .with(PointStyle {
        radius: Dimension::Pixels(1.0),
        ..Default::default()
      })
      .with(Selected)
      .build();

    Transition::ChangeState(Box::new(PlacingStart::new(temp_point)))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    _event_args: &MouseEventArgs,
  ) -> Transition {
    ctx.suppress_redraw();
    Transition::DoNothing
  }
}

#[derive(Debug)]
struct PlacingStart {
  temp_point: Entity,
}

impl PlacingStart {
  fn new(temp_point: Entity) -> Self {
    PlacingStart { temp_point }
  }
}

impl State for PlacingStart {
  fn on_mouse_up(&mut self, ctx: &mut dyn ApplicationContext, args: &MouseEventArgs) -> Transition {
    log::debug!("PlacingStart on_mouse_up called");

    // make sure nothing else is selected
    ctx.unselect_all();

    let layer = ctx.default_layer();

    // create a point and automatically mark it as selected
    let temp_line = ctx
      .world_mut()
      .create_entity()
      .with(DrawingObject {
        geometry: Geometry::Line(Line::new(args.location, args.location)),
        layer,
      })
      .with(Selected)
      .build();

    let _ = ctx.world_mut().delete_entity(self.temp_point);
    Transition::ChangeState(Box::new(WaitingToPlaceEnd::new(temp_line)))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    let world = ctx.world();
    let mut drawing_objects: WriteStorage<DrawingObject> = world.write_storage();

    let drawing_object = drawing_objects.get_mut(self.temp_point).unwrap();

    // we *know* this is a point. Instead of pattern matching or translating
    // the drawing object, we can just overwrite it with its new position.
    drawing_object.geometry = Geometry::Point(args.location);

    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    // make sure we clean up the temporary point.
    let _ = ctx.world_mut().delete_entity(self.temp_point);
  }
}

///////////////////////////////////////////////////////
/// End Vertex
// The base sub-state for [`AddPointMode`]. We're waiting for the user to click
/// so we can start adding a point to the canvas.
#[derive(Debug)]
struct WaitingToPlaceEnd {
  temp_line: Entity,
}

impl WaitingToPlaceEnd {
  fn new(temp_line: Entity) -> Self {
    WaitingToPlaceEnd { temp_line }
  }
}
impl State for WaitingToPlaceEnd {
  fn on_mouse_down(
    &mut self,
    _ctx: &mut dyn ApplicationContext,
    _args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("WaitingToPlace on_mouse_down called");

    Transition::ChangeState(Box::new(PlacingEnd::new(self.temp_line)))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("PlacingEnd on_mouse_move called");

    let world = ctx.world();
    let mut drawing_objects: WriteStorage<DrawingObject> = world.write_storage();

    let drawing_object = drawing_objects.get_mut(self.temp_line).unwrap();
    log::debug!("drawing_object.geometry {:?}", drawing_object.geometry);

    if let Geometry::Line(line) = drawing_object.geometry {
      drawing_object.geometry = Geometry::Line(Line::new(line.start, args.location));
    };

    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    // make sure we clean up the temporary line.
    let _ = ctx.world_mut().delete_entity(self.temp_line);
  }
}

#[derive(Debug)]
struct PlacingEnd {
  temp_line: Entity,
}

impl PlacingEnd {
  fn new(temp_line: Entity) -> Self {
    PlacingEnd { temp_line }
  }
}

impl State for PlacingEnd {
  fn on_mouse_up(
    &mut self,
    _ctx: &mut dyn ApplicationContext,
    _args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("PlacingEnd on_mouse_up called");

    Transition::ChangeState(Box::new(WaitingToPlaceStart::default()))
  }

  fn on_mouse_move(
    &mut self,
    ctx: &mut dyn ApplicationContext,
    args: &MouseEventArgs,
  ) -> Transition {
    log::debug!("PlacingEnd on_mouse_move called");

    let world = ctx.world();
    let mut drawing_objects: WriteStorage<DrawingObject> = world.write_storage();

    let drawing_object = drawing_objects.get_mut(self.temp_line).unwrap();

    if let Geometry::Line(line) = drawing_object.geometry {
      drawing_object.geometry = Geometry::Line(Line::new(line.start, args.location));
    };

    Transition::DoNothing
  }

  fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
    // make sure we clean up the temporary line.
    let _ = ctx.world_mut().delete_entity(self.temp_line);
  }
}
