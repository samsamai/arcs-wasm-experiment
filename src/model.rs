use arcs::{
  components::{
    layer::LayerType, CursorPosition, Dimension, DrawingObject, Geometry, GridStyle, Layer, Name,
    PointStyle,
  },
  euclid::{Length, Point2D, Size2D},
  piet::Color,
  primitives::Grid,
  specs::prelude::*,
  systems::deleter::Deleter,
  systems::draw::Draw,
  systems::mover::Mover,
  systems::snapper::Snapper,
  window::Window,
  CanvasSpace,
};

use super::keyboard_event_args::KeyboardEventArgs;
use super::msg::ButtonType;
use super::Props;

use super::modes::{ApplicationContext, Idle, MouseButtons, MouseEventArgs, State, Transition};

pub struct Model {
  pub world: World,
  pub window: Window,
  pub default_layer: Entity,
  pub system_layer: Entity,
  pub canvas_size: Size2D<f64, CanvasSpace>,
  pub current_state: Box<dyn State>,
  pub pointer: Entity,
  pub grid: Entity,
  pub command: Entity,
  pub dispatcher: Dispatcher<'static, 'static>,
  pub snap: bool,
}

impl Model {
  pub fn new(props: Props) -> Model {
    let mut world = World::new();
    arcs::components::register(&mut world);
    let builder = world.create_entity().with(PointStyle {
      radius: Dimension::Pixels(3.0),
      ..Default::default()
    });
    let default_layer = Layer::create(builder, Name::new("default_layer"), Layer::default());

    let builder = world.create_entity().with(GridStyle {
      stroke: Color::rgb8(0x94, 0x94, 0x94),
      width: Dimension::Pixels(0.1),
    });

    let system_layer = Layer::create(
      builder,
      Name::new("system_layer"),
      Layer {
        z_level: usize::MIN,
        visible: true,
        layer_type: LayerType::System,
      },
    );

    let window = Window::create(&mut world);
    window
      .style_mut(&mut world.write_storage())
      .background_colour = Color::rgb8(0xff, 0xcc, 0xcb);

    let pointer = world.create_entity().build();

    let _cursor_position = world.insert(CursorPosition::default());

    let grid = Grid::new(Length::new(20.), false);
    let grid = world
      .create_entity()
      .with(DrawingObject {
        geometry: Geometry::Grid(grid),
        layer: system_layer,
      })
      .build();

    let command = world.create_entity().with(Name::new("command")).build();

    let dispatcher = DispatcherBuilder::new()
      .with(Snapper, "snapper", &[])
      .with(Draw, "draw", &["snapper"])
      .with(Deleter, "deleter", &[])
      .with(Mover, "mover", &["snapper"])
      .build();

    Model {
      world,
      window,
      default_layer,
      system_layer,
      canvas_size: Size2D::new(props.width, props.height),
      current_state: Box::new(Idle::default()),
      grid: grid,
      pointer: pointer,
      command: command,
      dispatcher: dispatcher,
      snap: false,
    }
  }

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
        pointer: self.pointer,
        grid: self.grid,
        command: self.command,
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
    match args {
      ButtonType::Snap => {
        self.snap = !self.snap;

        let entities: Entities = self.world.entities();
        let mut drawing_objects: WriteStorage<DrawingObject> = self.world.write_storage();
        for (_entity, mut drawing_object) in (&entities, &mut drawing_objects).join() {
          if let Geometry::Grid(mut grid) = drawing_object.geometry {
            grid.snap = self.snap;
            drawing_object.geometry = Geometry::Grid(grid);
          };
        }
      }
      _ => (),
    }

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
  pointer: Entity,
  grid: Entity,
  command: Entity,
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

  fn pointer(&self) -> Entity {
    self.pointer
  }

  fn grid(&self) -> Entity {
    self.grid
  }
  fn command(&self) -> Entity {
    self.command
  }
}
