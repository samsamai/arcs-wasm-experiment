use crate::modes::{
    AddArcMode, AddLineMode, ApplicationContext, Idle, KeyboardEventArgs, MouseEventArgs, State,
    Transition, VirtualKeyCode,
};

use crate::msg::ButtonType;
use arcs::components::{AddPoint, CursorPosition, Delete, DrawingObject, Geometry, Selected};
use arcs::specs::prelude::*;
use arcs::specs::WorldExt;

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
            ButtonType::Select => Transition::ChangeState(Box::new(Idle::default())),
            ButtonType::Snap => Transition::DoNothing,
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
        let command_entity = ctx.command();
        {
            let mut storage: WriteStorage<AddPoint> = ctx.world_mut().write_storage();
            let _ = storage.insert(command_entity, AddPoint { layer });
        }
        let mut cursor_position = ctx.world_mut().write_resource::<CursorPosition>();
        cursor_position.location = args.location;

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
        ctx: &mut dyn ApplicationContext,
        _args: &MouseEventArgs,
    ) -> Transition {
        log::debug!("PlacingPoint on_mouse_up called");

        ctx.unselect_all();

        Transition::ChangeState(Box::new(WaitingToPlace::default()))
    }

    fn on_mouse_move(
        &mut self,
        ctx: &mut dyn ApplicationContext,
        args: &MouseEventArgs,
    ) -> Transition {
        let mut cursor_position = ctx.world_mut().write_resource::<CursorPosition>();
        cursor_position.location = args.location;

        Transition::DoNothing
    }

    fn on_cancelled(&mut self, ctx: &mut dyn ApplicationContext) {
        // make sure we clean up the temporary point.
        let command_entity = ctx.command();
        let mut storage: WriteStorage<Delete> = ctx.world_mut().write_storage();
        let _ = storage.insert(command_entity, Delete {});
    }
}
