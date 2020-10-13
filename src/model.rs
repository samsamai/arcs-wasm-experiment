use arcs::{
  components::{Dimension, Layer, Name, PointStyle},
  euclid::{Point2D, Size2D},
  piet::Color,
  specs::prelude::*,
  window::Window,
  CanvasSpace,
};

use super::keyboard_event_args::KeyboardEventArgs;
use super::msg::ButtonType;

use super::modes::{ApplicationContext, Idle, MouseButtons, MouseEventArgs, State, Transition};

pub struct Model {
  pub world: World,
  pub window: Window,
  pub default_layer: Entity,
  pub canvas_size: Size2D<f64, CanvasSpace>,
  pub current_state: Box<dyn State>,
}

impl Default for Model {
  fn default() -> Model {
    let mut world = World::new();
    arcs::components::register(&mut world);
    let builder = world.create_entity().with(PointStyle {
      radius: Dimension::Pixels(3.0),
      ..Default::default()
    });
    let default_layer = Layer::create(builder, Name::new("default"), Layer::default());

    let window = Window::create(&mut world);
    window
      .style_mut(&mut world.write_storage())
      .background_colour = Color::rgb8(0xff, 0xcc, 0xcb);

    Model {
      world,
      window,
      default_layer,
      canvas_size: Size2D::new(300.0, 150.0),
      current_state: Box::new(Idle::default()),
    }
  }
}

impl Model {
  fn handle_event<F>(&mut self, handler: F) -> bool
  where
    F: FnOnce(&mut dyn State, &mut Context<'_>) -> Transition,
  {
    let mut suppress_redraw = false;
    let transition = handler(
      &mut *self.current_state,
      &mut Context {
        world: &mut self.world,
        window: &mut self.window,
        default_layer: self.default_layer,
        suppress_redraw: &mut suppress_redraw,
      },
    );
    self.handle_transition(transition);
    !suppress_redraw
  }

  pub fn on_mouse_down(&mut self, cursor: Point2D<f64, CanvasSpace>) -> bool {
    let args = self.mouse_event_args(cursor);
    log::debug!("[ON_MOUSE_DOWN] {:?}, {:?}", args, self.current_state);
    self.handle_event(|state, ctx| state.on_mouse_down(ctx, &args))
  }

  pub fn on_mouse_up(&mut self, cursor: Point2D<f64, CanvasSpace>) -> bool {
    let args = self.mouse_event_args(cursor);
    log::debug!("[ON_MOUSE_UP] {:?}, {:?}", args, self.current_state);
    self.handle_event(|state, ctx| state.on_mouse_up(ctx, &args))
  }

  pub fn on_mouse_move(&mut self, cursor: Point2D<f64, CanvasSpace>) -> bool {
    let args = self.mouse_event_args(cursor);
    self.handle_event(|state, ctx| state.on_mouse_move(ctx, &args))
  }

  pub fn on_key_pressed(&mut self, args: KeyboardEventArgs) -> bool {
    log::debug!("[ON_KEY_PRESSED] {:?}, {:?}", args, self.current_state);
    self.handle_event(|state, ctx| state.on_key_pressed(ctx, &args))
  }

  pub fn on_button_clicked(&mut self, args: ButtonType) -> bool {
    log::debug!("[ON_BUTTON_CLICKED] {:?}, {:?}", args, self.current_state);
    self.handle_event(|state, ctx| state.on_button_clicked(ctx, &args))
  }

  fn handle_transition(&mut self, transition: Transition) {
    match transition {
      Transition::ChangeState(new_state) => {
        log::debug!("Changing state {:?} => {:?}", self.current_state, new_state);
        self.current_state = new_state
      }
      Transition::DoNothing => {}
    }
  }

  fn mouse_event_args(&self, cursor: Point2D<f64, CanvasSpace>) -> MouseEventArgs {
    let viewports = self.world.read_storage();
    let viewport = self.window.viewport(&viewports);
    let location = arcs::window::to_drawing_coordinates(cursor, viewport, self.canvas_size);

    MouseEventArgs {
      location,
      cursor,
      button_state: MouseButtons::LEFT_BUTTON,
    }
  }
}

/// A temporary struct which presents a "view" of [`Model`] which can be used
/// as a [`ApplicationContext`].
struct Context<'model> {
  world: &'model mut World,
  window: &'model mut Window,
  default_layer: Entity,
  suppress_redraw: &'model mut bool,
}

impl<'model> ApplicationContext for Context<'model> {
  fn world(&self) -> &World {
    &self.world
  }

  fn world_mut(&mut self) -> &mut World {
    &mut self.world
  }

  fn viewport(&self) -> Entity {
    self.window.0
  }

  fn default_layer(&self) -> Entity {
    self.default_layer
  }

  fn suppress_redraw(&mut self) {
    *self.suppress_redraw = true;
  }
}
