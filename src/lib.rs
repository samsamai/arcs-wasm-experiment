use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent};

use arcs::{
    components::{
        Dimension, DrawingObject, Geometry, Layer, LineStyle, Name, PointStyle, Viewport,
    },
    euclid::{Point2D, Scale, Size2D},
    piet::Color,
    piet_web::WebRenderContext,
    specs::prelude::*,
    window::Window,
    CanvasSpace, Length, Line, Point,
};
use log::Level;

use seed::{prelude::*, *};
use std::f64::consts::PI;

use modes::{
    ApplicationContext, Idle, KeyboardEventArgs, MouseButtons, MouseEventArgs, State, Transition,
    VirtualKeyCode,
};
use std::convert::TryFrom;

pub mod modes;
mod utils;

const CANVAS_ID: &str = "canvas";

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/////////////////////////////////////////////////////////////////////////////////////////
///
///
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
/////////////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
pub fn run() {
    #[cfg(feature = "console_error_panic_hook")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    env_logger::init();

    // Create a world and add some items to it
    let mut world = World::new();

    // make sure we register all components
    arcs::components::register(&mut world);

    let layer = Layer::create(
        world.create_entity(),
        Name::new("default"),
        Layer {
            z_level: 0,
            visible: true,
        },
    );

    // add a green dot to the world
    world
        .create_entity()
        .with(DrawingObject {
            geometry: Geometry::Point(Point::new(20.0, 0.0)),
            layer,
        })
        .with(PointStyle {
            radius: Dimension::Pixels(50.0),
            colour: Color::rgb8(0x00, 0xff, 0),
        })
        .build();
    world
        .create_entity()
        .with(DrawingObject {
            geometry: Geometry::Point(Point::new(-20.0, 0.0)),
            layer,
        })
        .with(PointStyle {
            radius: Dimension::Pixels(50.0),
            colour: Color::rgb8(0x00, 0x00, 0xff),
        })
        .build();
    // and a red hexagon
    let angles = (0..7).map(|i| i as f64 * 2.0 * PI / 6.0);
    let radius = 50.0;
    for (start_angle, end_angle) in angles.clone().zip(angles.clone().skip(1)) {
        let start = Point::new(radius * start_angle.cos(), radius * start_angle.sin());
        let end = Point::new(radius * end_angle.cos(), radius * end_angle.sin());

        world
            .create_entity()
            .with(DrawingObject {
                geometry: Geometry::Line(Line::new(start, end)),
                layer,
            })
            .with(LineStyle {
                width: Dimension::DrawingUnits(Length::new(5.0)),
                stroke: Color::rgb8(0xff, 0, 0),
            })
            .build();
    }

    // now we've added some objects to the world we can start rendering
    let arcs_window = Window::create(&mut world);

    // set the viewport and background colour
    *arcs_window.viewport_mut(&mut world.write_storage()) = Viewport {
        centre: Point::zero(),
        pixels_per_drawing_unit: Scale::new(5.0),
    };
    arcs_window
        .style_mut(&mut world.write_storage())
        .background_colour = Color::WHITE;

    let browser_window = window().unwrap();

    let canvas_element = browser_window
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .expect("Canvas element not found");

    let canvas = canvas_element.dyn_into::<HtmlCanvasElement>().unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let dpr = browser_window.device_pixel_ratio();
    canvas.set_width((canvas.offset_width() as f64 * dpr) as u32);
    canvas.set_height((canvas.offset_height() as f64 * dpr) as u32);
    let _ = context.scale(dpr, dpr);

    let w = canvas.offset_width() as f64;
    let h = canvas.offset_height() as f64;
    console::log_2(&w.into(), &h.into());

    let arcs_context = WebRenderContext::new(context, browser_window);

    let mut system = arcs_window.render_system(arcs_context, Size2D::new(w, h));
    RunNow::setup(&mut system, &mut world);
    RunNow::run_now(&mut system, &world);
}
pub struct Model {
    world: World,
    window: Window,
    default_layer: Entity,
    canvas_size: Size2D<f64, CanvasSpace>,
    current_state: Box<dyn State>,
}

impl Default for Model {
    fn default() -> Model {
        let mut world = World::new();
        arcs::components::register(&mut world);
        let builder = world.create_entity().with(PointStyle {
            radius: Dimension::Pixels(10.0),
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

    fn on_mouse_down(&mut self, cursor: Point2D<f64, CanvasSpace>) -> bool {
        let args = self.mouse_event_args(cursor);
        log::debug!("[ON_MOUSE_DOWN] {:?}, {:?}", args, self.current_state);
        self.handle_event(|state, ctx| state.on_mouse_down(ctx, &args))
    }

    fn on_mouse_up(&mut self, cursor: Point2D<f64, CanvasSpace>) -> bool {
        let args = self.mouse_event_args(cursor);
        log::debug!("[ON_MOUSE_UP] {:?}, {:?}", args, self.current_state);
        self.handle_event(|state, ctx| state.on_mouse_up(ctx, &args))
    }

    fn on_mouse_move(&mut self, cursor: Point2D<f64, CanvasSpace>) -> bool {
        let args = self.mouse_event_args(cursor);
        self.handle_event(|state, ctx| state.on_mouse_move(ctx, &args))
    }

    fn on_key_pressed(&mut self, args: KeyboardEventArgs) -> bool {
        log::debug!("[ON_KEY_PRESSED] {:?}, {:?}", args, self.current_state);
        self.handle_event(|state, ctx| state.on_key_pressed(ctx, &args))
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

fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    orders
        .after_next_render(|_| Msg::Rendered)
        .after_next_render(|_| Msg::WindowResized);

    AfterMount::new(Model::default())
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    log::trace!("Handling {:?}", msg);

    let needs_render = match msg {
        Msg::Rendered => true,
        Msg::MouseDown(cursor) => model.on_mouse_down(cursor),
        Msg::MouseUp(cursor) => model.on_mouse_up(cursor),
        Msg::MouseMove(cursor) => model.on_mouse_move(cursor),
        Msg::KeyPressed(args) => model.on_key_pressed(args),
        Msg::WindowResized => {
            if let Some(parent_size) =
                seed::canvas(CANVAS_ID).and_then(|canvas| parent_size(&canvas))
            {
                log::debug!("Changing the canvas to {}", parent_size);
                model.canvas_size = parent_size;
            }

            true
        }
    };

    if needs_render {
        if let Some(canvas) = seed::canvas(CANVAS_ID) {
            draw(&canvas, model);
        }
    }
}
fn draw(canvas: &HtmlCanvasElement, model: &mut Model) {
    let mut canvas_ctx = seed::canvas_context_2d(&canvas);
    let browser_window = seed::window();
    let ctx = WebRenderContext::new(canvas_ctx, browser_window);

    let mut system = model.window.render_system(ctx, model.canvas_size);
    RunNow::setup(&mut system, &mut model.world);
    RunNow::run_now(&mut system, &model.world);
}

fn parent_size(element: &HtmlElement) -> Option<Size2D<f64, CanvasSpace>> {
    let window = seed::window();
    let height =
        window.inner_height().ok()?.as_f64()? - f64::try_from(element.offset_top()).ok()?;
    let width = window.inner_width().ok()?.as_f64()?;
    log::debug!("parent size is {}x{}", height, width);

    Some(Size2D::new(
        f64::try_from(width).ok()?,
        f64::try_from(height).ok()?,
    ))
}

fn canvas_location(ev: MouseEvent) -> Point2D<f64, CanvasSpace> {
    let x = ev.offset_x().into();
    let y = ev.offset_y().into();

    Point2D::new(x, y)
}

fn view(model: &Model) -> impl View<Msg> {
    log::debug!("view called");

    div![div![
        attrs![ At::Class => "canvas-container" ],
        style! {
            St::Width => "100%",
            St::Height => "100%",
            St::OverflowY => "hidden",
            St::OverflowX => "hidden",
        },
        canvas![
            attrs![
                At::Id => CANVAS_ID,
                At::Width => model.canvas_size.width,
                At::Height => model.canvas_size.height,
                At::TabIndex => "1",
            ],
            mouse_ev(Ev::MouseDown, |e| Msg::MouseDown(canvas_location(e))),
            mouse_ev(Ev::MouseUp, |e| Msg::MouseUp(canvas_location(e))),
            mouse_ev(Ev::MouseMove, |e| Msg::MouseMove(canvas_location(e))),
            keyboard_ev(Ev::KeyDown, Msg::from_key_press)
        ],
    ]]
}

pub fn window_events(_model: &Model) -> Vec<Listener<Msg>> {
    vec![simple_ev(Ev::Resize, Msg::WindowResized)]
}

#[wasm_bindgen(start)]
pub fn render() {
    console_log::init_with_level(Level::Debug).expect("Unable to initialize the log");
    log::debug!("render called");

    seed::App::builder(update, view)
        .after_mount(after_mount)
        .window_events(window_events)
        .build_and_start();
}
