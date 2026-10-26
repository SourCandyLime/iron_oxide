use crate::{canvas::Canvas, input::Input, time::Time, GameContext};

pub enum StateAction<Ctx: GameContext> {
    None,
    Pop,
    Push(Box<dyn GameState<Ctx>>),
    Replace(Box<dyn GameState<Ctx>>),
    Exit,
}

pub trait GameState<Ctx: GameContext> {
    fn new() -> Self
    where
        Self: Sized;

    fn start(&mut self, ctx: &mut Ctx) {}

    fn update(
        &mut self,
        ctx: &mut Ctx,
        time: &Time,
        input: &Input,
    ) -> StateAction<Ctx> {
        StateAction::None
    }

    fn render(
        &mut self,
        ctx: &mut Ctx,
        canvas: &mut Canvas,
    ) {}

    fn end(&mut self, ctx: &mut Ctx) {}

    fn render_below(&self) -> bool {
        false
    }
}