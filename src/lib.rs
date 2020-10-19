use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlElement, MouseEvent};

use arcs::{
    euclid::{Point2D, Size2D},
    piet_web::WebRenderContext,
    specs::prelude::*,
    CanvasSpace,
};
use log::Level;

use seed::{prelude::*, *};

use crate::model::Model;
use msg::ButtonType;
use std::convert::TryFrom;

mod keyboard_event_args;
mod model;
mod modes;
mod msg;
mod utils;

const CANVAS_ID: &str = "canvas";

/////////////////////////////////////////////////////////////////////////////////////////
///
#[wasm_bindgen(start)]
pub fn render() {
    console_log::init_with_level(Level::Debug).expect("Unable to initialize the log");
    log::debug!("render called");

    seed::App::builder(update, view)
        .after_mount(after_mount)
        .window_events(window_events)
        .build_and_start();
}

fn after_mount(_: Url, orders: &mut impl Orders<msg::Msg>) -> AfterMount<Model> {
    orders
        .after_next_render(|_| msg::Msg::Rendered)
        .after_next_render(|_| msg::Msg::WindowResized);

    AfterMount::new(Model::default())
}

fn update(msg: msg::Msg, model: &mut Model, _orders: &mut impl Orders<msg::Msg>) {
    log::trace!("Handling {:?}", msg);

    let needs_render = match msg {
        msg::Msg::Rendered => true,
        msg::Msg::MouseDown(cursor) => model.on_mouse_down(cursor),
        msg::Msg::MouseUp(cursor) => model.on_mouse_up(cursor),
        msg::Msg::MouseMove(cursor) => model.on_mouse_move(cursor),
        msg::Msg::KeyPressed(args) => model.on_key_pressed(args),
        msg::Msg::ButtonClicked(args) => model.on_button_clicked(args),
        msg::Msg::WindowResized => {
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
    model.dispatcher.dispatch(&model.world);
    model.world.maintain();

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

fn view(model: &Model) -> Node<msg::Msg> {
    div![
        nav![
            C!["tools level"],
            div![
                C!["level-item has-text-centered"],
                div![
                    C!["field has-addons"],
                    p![
                        C!["control"],
                        button![
                            C![format!(
                                "button is-light{}",
                                if (*model.current_state)
                                    .as_any()
                                    .is::<modes::add_point_mode::AddPointMode>()
                                {
                                    " is-inverted is-active"
                                } else {
                                    ""
                                }
                            ),],
                            span![C!["icon is-small"], i![C!["fas fa-plus"]]],
                            span!["Point"],
                            ev(Ev::Click, |_| msg::Msg::ButtonClicked(ButtonType::Point)),
                        ]
                    ],
                    p![
                        C!["control"],
                        button![
                            C![format!(
                                "button is-light{}",
                                if (*model.current_state)
                                    .as_any()
                                    .is::<modes::add_line_mode::AddLineMode>()
                                {
                                    " is-inverted is-active"
                                } else {
                                    ""
                                }
                            ),],
                            span![C!["icon is-small"], i![C!["fas fa-pen-fancy"]]],
                            span!["Line"],
                            ev(Ev::Click, |_| msg::Msg::ButtonClicked(ButtonType::Line)),
                        ]
                    ]
                ]
            ]
        ],
        div![
            attrs![ At::Class => "canvas-container" ],
            style! {
                St::Width => "100%",
                St::Height => "100%",
                St::OverflowY => "hidden",
                St::OverflowX => "hidden",
                St::Cursor => {&(*model.current_state.get_cursor())}
            },
            canvas![
                attrs![
                    At::Id => CANVAS_ID,
                    At::Width => model.canvas_size.width,
                    At::Height => model.canvas_size.height,
                    At::TabIndex => "1",
                ],
                mouse_ev(Ev::MouseDown, |e| msg::Msg::MouseDown(canvas_location(e))),
                mouse_ev(Ev::MouseUp, |e| msg::Msg::MouseUp(canvas_location(e))),
                mouse_ev(Ev::MouseMove, |e| msg::Msg::MouseMove(canvas_location(e))),
                keyboard_ev(Ev::KeyDown, msg::Msg::from_key_press)
            ],
        ]
    ]
}

fn canvas_location(ev: MouseEvent) -> Point2D<f64, CanvasSpace> {
    let x = ev.offset_x().into();
    let y = ev.offset_y().into();

    Point2D::new(x, y)
}

fn window_events(_model: &Model) -> Vec<EventHandler<msg::Msg>> {
    vec![simple_ev(Ev::Resize, msg::Msg::WindowResized)]
}

///////////////////////////////////////////////////////////////////////////////////
/// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
