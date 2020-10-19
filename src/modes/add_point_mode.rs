use crate::modes::{
    AddArcMode, AddLineMode, ApplicationContext, Idle, KeyboardEventArgs, MouseEventArgs, State,
    Transition, VirtualKeyCode,
};

use crate::msg::ButtonType;
use arcs::components::{DrawingObject, Geometry, Selected};
use arcs::specs::prelude::*;

#[derive(Debug)]
pub struct AddPointMode {
    nested: Box<dyn State>,
}

impl AddPointMode {
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

impl State for AddPointMode {
    fn on_mouse_down(
        &mut self,
        ctx: &mut dyn ApplicationContext,
        args: &MouseEventArgs,
    ) -> Transition {
        let trans = self.nested.on_mouse_down(ctx, args);
        self.handle_transition(trans);
        Transition::DoNothing
    }

    fn on_mouse_up(
        &mut self,
        ctx: &mut dyn ApplicationContext,
        args: &MouseEventArgs,
    ) -> Transition {
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
        self.nested = Box::new(WaitingToPlace::default());
    }

    fn on_button_clicked(
        &mut self,
        _ctx: &mut dyn ApplicationContext,
        event_args: &ButtonType,
    ) -> Transition {
        match event_args {
            ButtonType::Arc => Transition::ChangeState(Box::new(AddArcMode::default())),
            ButtonType::Point => Transition::ChangeState(Box::new(AddPointMode::default())),
            ButtonType::Line => Transition::ChangeState(Box::new(AddLineMode::default())),
        }
    }
}

impl Default for AddPointMode {
    fn default() -> AddPointMode {
        AddPointMode {
            nested: Box::new(WaitingToPlace::default()),
        }
    }
}

/// The base sub-state for [`AddPointMode`]. We're waiting for the user to click
/// so we can start adding a point to the canvas.
#[derive(Debug, Default)]
struct WaitingToPlace;

impl State for WaitingToPlace {
    fn on_mouse_down(
        &mut self,
        ctx: &mut dyn ApplicationContext,
        args: &MouseEventArgs,
    ) -> Transition {
        log::debug!("WaitingToPlace on_mouse_down called");

        // make sure nothing else is selected
        ctx.unselect_all();

        let layer = ctx.default_layer();
        let effective_location = ctx.effective_location(args.location);
        let command_entity = ctx.command();
        ctx.add_command(command_entity, args, layer);
        // let effective_location = args.location;

        // let temp_point = ctx
        //     .world_mut()
        //     .create_entity()
        //     .with(DrawingObject {
        //         geometry: Geometry::Point(effective_location),
        //         layer,
        //     })
        //     .with(Selected)
        //     .build();

        Transition::ChangeState(Box::new(PlacingPoint {}))
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
struct PlacingPoint;

impl State for PlacingPoint {
    fn on_mouse_up(
        &mut self,
        _ctx: &mut dyn ApplicationContext,
        _args: &MouseEventArgs,
    ) -> Transition {
        log::debug!("PlacingPoint on_mouse_up called");

        // We "commit" the change by leaving the temporary point where it is
        Transition::ChangeState(Box::new(WaitingToPlace::default()))
    }

    fn on_mouse_move(
        &mut self,
        ctx: &mut dyn ApplicationContext,
        args: &MouseEventArgs,
    ) -> Transition {
        // let world = ctx.world();
        // let mut drawing_objects: WriteStorage<DrawingObject> = world.write_storage();

        // let drawing_object = drawing_objects.get_mut(self.temp_point).unwrap();
        // let effective_location = ctx.effective_location(args.location);
        // // let effective_location = args.location;

        // // we *know* this is a point. Instead of pattern matching or translating
        // // the drawing object, we can just overwrite it with its new position.
        // drawing_object.geometry = Geometry::Point(effective_location);

        Transition::DoNothing
    }

    fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
        // make sure we clean up the temporary point.
        // let _ = ctx.world_mut().delete_entity(self.temp_point);
    }
}
