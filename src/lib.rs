#![recursion_limit = "512"]

use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlCanvasElement, HtmlElement, MouseEvent};

use arcs::{
    euclid::{Point2D, Size2D},
    piet_web::WebRenderContext,
    specs::prelude::*,
    CanvasSpace,
};
use log::Level;

// use seed::{prelude::*, *};

use crate::model::Model;
use msg::ButtonType;
use std::convert::TryFrom;
use wasm_bindgen::JsCast;

mod keyboard_event_args;
mod model;
mod modes;
mod msg;
mod utils;

const CANVAS_ID: &str = "canvas";

/////////////////////////////////////////////////////////////////////////////////////////
use yew::prelude::*;
use yew::services::resize::ResizeTask;
use yew::services::ResizeService;
use yew::utils::*;
use yew::Component;

#[wasm_bindgen(start)]
pub fn run_app() {
    console_log::init_with_level(Level::Debug).expect("Unable to initialize the log");

    log::debug!("run_app called");

    App::<Main>::new().mount_to_body();
}

struct Main {
    link: ComponentLink<Self>,
    model: Model,
    _resize_task: ResizeTask,
}

// enum Msg {
//     AddOne,
// }

impl Component for Main {
    type Message = msg::Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::debug!("main created");
        let _resize_task =
            ResizeService::new().register(link.callback(|_wd| msg::Msg::WindowResized));

        Self {
            link,
            model: Model::default(),
            _resize_task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        log::trace!("Handling {:?}", msg);

        let needs_render = match msg {
            msg::Msg::Rendered => true,
            msg::Msg::MouseDown(cursor) => self.model.on_mouse_down(cursor),
            msg::Msg::MouseUp(cursor) => self.model.on_mouse_up(cursor),
            msg::Msg::MouseMove(cursor) => self.model.on_mouse_move(cursor),
            msg::Msg::KeyPressed(args) => self.model.on_key_pressed(args),
            msg::Msg::ButtonClicked(args) => self.model.on_button_clicked(args),
            msg::Msg::WindowResized => self.resize(),
        };

        if needs_render {
            if let Some(canvas) = self.canvas(CANVAS_ID) {
                self.draw(&canvas);
            };
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        log::debug!("change called");

        false
    }
}

    fn rendered(&mut self, first_render: bool) {
        // if first_render {
        //     self.resize();
        //     if let Some(canvas) = self.canvas(CANVAS_ID) {
        //         let canvas_ctx = self.canvas_context_2d(&canvas);
        //         let browser_window = window();
        //         let ctx = WebRenderContext::new(canvas_ctx, browser_window);

        //         let mut system = self.model.window.render_system(ctx, self.model.canvas_size);
        //         RunNow::setup(&mut system, &mut self.model.world);
        //         RunNow::run_now(&mut system, &self.model.world);
        //     }
        //     log::debug!("resize task ");
        //     self.update(msg::Msg::WindowResized);
        // }
    }

    fn view(&self) -> Html {
        html! {
                <div>
                    <nav class="tools level">
                        <div class="level-item has-text-centered">
                            <div class="field has-addons">
                                <p class="control"
                                    onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Point))
                                >{self.view_point_btn()}</p>
                                <p class="control"
                                    onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Line))
                                >{self.view_line_btn()}</p>
                            </div>
                        </div>
                    </nav>
                    <div class="canvas-container">
                        <canvas id="canvas" width={self.model.canvas_size.width}
                        height={self.model.canvas_size.height} tabindex=1
                            onmousedown=self.link.callback(|e| msg::Msg::MouseDown(canvas_location(e)))
                            onmouseup=self.link.callback(|e| msg::Msg::MouseUp(canvas_location(e)))
                            onmousemove=self.link.callback(|e| msg::Msg::MouseMove(canvas_location(e)))
                            onkeydown=self.link.callback(msg::Msg::from_key_press)
                        ></canvas>
                    </div>
                </div>
        }
    }
}

impl Main {
    fn view_point_btn(&self) -> Html {
        let classes = if (*self.model.current_state)
            .as_any()
            .is::<modes::add_point_mode::AddPointMode>()
        {
            "button is-light is-inverted is-active"
        } else {
            "button is-light"
        };

        html! {
            <button class={classes}
                onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Point))
            >
                <span class="icon is-small">
                    <i class="fas fa-plus"></i>
                </span>
                <span>{"Point"}</span>
            </button>
        }
    }

    fn view_line_btn(&self) -> Html {
        let classes = if (*self.model.current_state)
            .as_any()
            .is::<modes::add_line_mode::AddLineMode>()
        {
            "button is-light is-inverted is-active"
        } else {
            "button is-light"
        };

        html! {
            <button class={classes}>
                <span class="icon is-small">
                    <i class="fas fa-pen-fancy"></i>
                </span>
                <span>{"Line"}</span>
            </button>
        }
    }

    fn resize(&mut self) -> bool {
        if let Some(parent_size) = self
            .canvas(CANVAS_ID)
            .and_then(|canvas| self.parent_size(&canvas))
        {
            log::debug!("Changing the canvas to {}", parent_size);
            self.model.canvas_size = parent_size;
            if let Some(canvas) = self.canvas(CANVAS_ID) {
                let canvas_ctx = self.canvas_context_2d(&canvas);
                let browser_window = window();
                let ctx = WebRenderContext::new(canvas_ctx, browser_window);

                let mut system = self.model.window.render_system(ctx, self.model.canvas_size);
                log::debug!("RunNow");

                // RunNow::setup(&mut system, &mut self.model.world);
                RunNow::run_now(&mut system, &self.model.world);
            }
        }

        true
    }

    fn canvas(&self, id: &str) -> Option<web_sys::HtmlCanvasElement> {
        document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    }

    /// Convenience function to access the `web_sys::CanvasRenderingContext2d`.
    pub fn canvas_context_2d(
        &self,
        canvas: &web_sys::HtmlCanvasElement,
    ) -> web_sys::CanvasRenderingContext2d {
        canvas
            .get_context("2d")
            .expect("Problem getting canvas context")
            .expect("The canvas context is empty")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("Problem casting as web_sys::CanvasRenderingContext2d")
    }

    fn draw(&mut self, canvas: &HtmlCanvasElement) {
        log::debug!("draw called");

        let canvas_ctx = self.canvas_context_2d(&canvas);
        let browser_window = window();
        let ctx = WebRenderContext::new(canvas_ctx, browser_window);
        log::debug!("draw called {:?}", self.model.canvas_size);

        let mut system = self.model.window.render_system(ctx, self.model.canvas_size);
        // RunNow::setup(&mut system, &mut self.model.world);
        RunNow::run_now(&mut system, &self.model.world);
    }

    fn parent_size(&self, element: &HtmlElement) -> Option<Size2D<f64, CanvasSpace>> {
        let window = window();
        let height =
            window.inner_height().ok()?.as_f64()? - f64::try_from(element.offset_top()).ok()?;
        let width = window.inner_width().ok()?.as_f64()?;
        log::debug!("parent size is {}x{}", height, width);

        Some(Size2D::new(
            f64::try_from(width).ok()?,
            f64::try_from(height).ok()?,
        ))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
///
// #[wasm_bindgen(start)]
// pub fn render() {
//     console_log::init_with_level(Level::Debug).expect("Unable to initialize the log");
//     log::debug!("render called");

//     seed::App::builder(update, view)
//         .after_mount(after_mount)
//         .window_events(window_events)
//         .build_and_start();
// }

// fn after_mount(_: Url, orders: &mut impl Orders<msg::Msg>) -> AfterMount<Model> {
//     orders
//         .after_next_render(|_| msg::Msg::Rendered)
//         .after_next_render(|_| msg::Msg::WindowResized);

//     AfterMount::new(Model::default())
// }

// fn update(msg: msg::Msg, model: &mut Model, _orders: &mut impl Orders<msg::Msg>) {
//     log::trace!("Handling {:?}", msg);

//     let needs_render = match msg {
//         msg::Msg::Rendered => true,
//         msg::Msg::MouseDown(cursor) => model.on_mouse_down(cursor),
//         msg::Msg::MouseUp(cursor) => model.on_mouse_up(cursor),
//         msg::Msg::MouseMove(cursor) => model.on_mouse_move(cursor),
//         msg::Msg::KeyPressed(args) => model.on_key_pressed(args),
//         msg::Msg::ButtonClicked(args) => model.on_button_clicked(args),
//         msg::Msg::WindowResized => {
//             if let Some(parent_size) =
//                 seed::canvas(CANVAS_ID).and_then(|canvas| parent_size(&canvas))
//             {
//                 log::debug!("Changing the canvas to {}", parent_size);
//                 model.canvas_size = parent_size;
//             }

//             true
//         }
//     };

//     if needs_render {
//         if let Some(canvas) = seed::canvas(CANVAS_ID) {
//             draw(&canvas, model);
//         }
//     }
// }
// fn draw(canvas: &HtmlCanvasElement, model: &mut Model) {
//     let canvas_ctx = seed::canvas_context_2d(&canvas);
//     let browser_window = seed::window();
//     let ctx = WebRenderContext::new(canvas_ctx, browser_window);

//     let mut system = model.window.render_system(ctx, model.canvas_size);
//     RunNow::setup(&mut system, &mut model.world);
//     RunNow::run_now(&mut system, &model.world);
// }

// fn parent_size(element: &HtmlElement) -> Option<Size2D<f64, CanvasSpace>> {
//     let window = seed::window();
//     let height =
//         window.inner_height().ok()?.as_f64()? - f64::try_from(element.offset_top()).ok()?;
//     let width = window.inner_width().ok()?.as_f64()?;
//     log::debug!("parent size is {}x{}", height, width);

//     Some(Size2D::new(
//         f64::try_from(width).ok()?,
//         f64::try_from(height).ok()?,
//     ))
// }

// fn view(model: &Model) -> Node<msg::Msg> {
//     div![
//         nav![
//             C!["tools level"],
//             div![
//                 C!["level-item has-text-centered"],
//                 div![
//                     C!["field has-addons"],
//                     p![
//                         C!["control"],
//                         button![
//                             C![format!(
//                                 "button is-light{}",
//                                 if (*model.current_state)
//                                     .as_any()
//                                     .is::<modes::add_point_mode::AddPointMode>()
//                                 {
//                                     " is-inverted is-active"
//                                 } else {
//                                     ""
//                                 }
//                             ),],
//                             span![C!["icon is-small"], i![C!["fas fa-plus"]]],
//                             span!["Point"],
//                             ev(Ev::Click, |_| msg::Msg::ButtonClicked(ButtonType::Point)),
//                         ]
//                     ],
//                     p![
//                         C!["control"],
//                         button![
//                             C![format!(
//                                 "button is-light{}",
//                                 if (*model.current_state)
//                                     .as_any()
//                                     .is::<modes::add_line_mode::AddLineMode>()
//                                 {
//                                     " is-inverted is-active"
//                                 } else {
//                                     ""
//                                 }
//                             ),],
//                             span![C!["icon is-small"], i![C!["fas fa-pen-fancy"]]],
//                             span!["Line"],
//                             ev(Ev::Click, |_| msg::Msg::ButtonClicked(ButtonType::Line)),
//                         ]
//                     ]
//                 ]
//             ]
//         ],
//         div![
//             attrs![ At::Class => "canvas-container" ],
//             style! {
//                 St::Width => "100%",
//                 St::Height => "100%",
//                 St::OverflowY => "hidden",
//                 St::OverflowX => "hidden",
//             },
//             canvas![
//                 attrs![
//                     At::Id => CANVAS_ID,
//                     At::Width => model.canvas_size.width,
//                     At::Height => model.canvas_size.height,
//                     At::TabIndex => "1",
//                 ],
//                 mouse_ev(Ev::MouseDown, |e| msg::Msg::MouseDown(canvas_location(e))),
//                 mouse_ev(Ev::MouseUp, |e| msg::Msg::MouseUp(canvas_location(e))),
//                 mouse_ev(Ev::MouseMove, |e| msg::Msg::MouseMove(canvas_location(e))),
//                 keyboard_ev(Ev::KeyDown, msg::Msg::from_key_press)
//             ],
//         ]
//     ]
// }

fn canvas_location(ev: MouseEvent) -> Point2D<f64, CanvasSpace> {
    let x = ev.offset_x().into();
    let y = ev.offset_y().into();

    Point2D::new(x, y)
}

// fn window_events(_model: &Model) -> Vec<EventHandler<msg::Msg>> {
//     vec![simple_ev(Ev::Resize, msg::Msg::WindowResized)]
// }

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
