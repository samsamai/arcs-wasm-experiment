#![recursion_limit = "512"]
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlElement, MouseEvent};

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

    let window = window();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap();
    App::<Main>::new().mount_to_body_with_props(Props { width, height });
}

struct Main {
    props: Props,
    link: ComponentLink<Self>,
    model: Model,
    _resize_task: ResizeTask,
}

#[derive(Copy, Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub width: f64,
    pub height: f64,
}

impl Component for Main {
    type Message = msg::Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let _resize_task =
            ResizeService::new().register(link.callback(|_wd| msg::Msg::WindowResized));

        Self {
            props,
            link,
            model: Model::new(props),
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.resize();
            if let Some(canvas) = self.canvas(CANVAS_ID) {
                let canvas_ctx = self.canvas_context_2d(&canvas);
                let browser_window = window();
                let ctx = WebRenderContext::new(canvas_ctx, browser_window);

                let mut system = self.model.window.render_system(ctx, self.model.canvas_size);
                RunNow::setup(&mut system, &mut self.model.world);
                RunNow::run_now(&mut system, &self.model.world);
            }
        }
    }

    fn view(&self) -> Html {
        html! {
                <div>
                    <nav class="tools level">
                        <div class="level-item has-text-centered">
                            <div class="field has-addons">
                                <p class="control"
                                    onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Select))
                                >{self.view_select_btn()}</p>
                                <p class="control"
                                    onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Point))
                                >{self.view_point_btn()}</p>
                                <p class="control"
                                    onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Line))
                                >{self.view_line_btn()}</p>
                                <p class="control"
                                    onclick=self.link.callback(|_| msg::Msg::ButtonClicked(ButtonType::Snap))
                                >{self.view_snap_btn()}</p>
                            </div>
                        </div>
                    </nav>
                    <div class="canvas-container" style={format!("cursor: {}", self.model.current_state.get_cursor())}>
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
    fn view_select_btn(&self) -> Html {
        let classes = if (*self.model.current_state)
            .as_any()
            .is::<modes::idle::Idle>()
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
                    <i class="fas fa-mouse-pointer"></i>
                </span>
                <span>{"Select"}</span>
            </button>
        }
    }

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

    fn view_snap_btn(&self) -> Html {
        let classes = "button is-light";

        let icon_class = if self.model.snap {
            "fas fa-toggle-on"
        } else {
            "fas fa-toggle-off"
        };

        html! {
            <button class={classes}>
                <span class="icon is-small">
                    <i class={icon_class}></i>
                </span>
                <span>{"Snap"}</span>
            </button>
        }
    }

    fn resize(&mut self) -> bool {
        if let Some(canvas) = self.canvas(CANVAS_ID) {
            self.model.canvas_size = self.parent_size(&canvas).unwrap();
            self.draw(&canvas);
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
        self.model.dispatcher.dispatch(&self.model.world);
        self.model.world.maintain();

        let canvas_ctx = self.canvas_context_2d(&canvas);
        let browser_window = window();
        let ctx = WebRenderContext::new(canvas_ctx, browser_window);

        let mut system = self.model.window.render_system(ctx, self.model.canvas_size);
        RunNow::setup(&mut system, &mut self.model.world);
        RunNow::run_now(&mut system, &self.model.world);
    }

    fn parent_size(&self, element: &HtmlElement) -> Option<Size2D<f64, CanvasSpace>> {
        let window = window();
        let height =
            window.inner_height().ok()?.as_f64()? - f64::try_from(element.offset_top()).ok()?;
        let width = window.inner_width().ok()?.as_f64()?;

        Some(Size2D::new(
            f64::try_from(width).ok()?,
            f64::try_from(height).ok()?,
        ))
    }
}

fn canvas_location(ev: MouseEvent) -> Point2D<f64, CanvasSpace> {
    let x = ev.offset_x().into();
    let y = ev.offset_y().into();

    Point2D::new(x, y)
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
