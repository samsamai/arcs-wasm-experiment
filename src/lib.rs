use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent};

use arcs::{
    euclid::{Point2D, Size2D},
    piet_web::WebRenderContext,
    specs::prelude::*,
    CanvasSpace,
};
use log::Level;

use keyboard_event_args::{KeyboardEventArgs, VirtualKeyCode};
use seed::{prelude::*, *};

// use modes::{
//     ApplicationContext, Idle, KeyboardEventArgs, MouseButtons, MouseEventArgs, State, Transition,
//     VirtualKeyCode,
// };
use crate::model::Model;
use std::convert::TryFrom;

mod keyboard_event_args;
pub mod model;
mod modes;
// use modes::*;
// pub mod modes;
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
    let canvas_ctx = seed::canvas_context_2d(&canvas);
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
