use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, HtmlCanvasElement};

use arcs::{
    components::{
        Dimension, DrawingObject, Geometry, Layer, LineStyle, Name, PointStyle, Viewport,
    },
    euclid::{Scale, Size2D},
    piet::Color,
    piet_web::WebRenderContext,
    specs::prelude::*,
    window::Window,
    Length, Line, Point,
};
use std::f64::consts::PI;

mod utils;

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
