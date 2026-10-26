use super::gamestate::{GameState, StateAction};
use crate::{
    canvas::Canvas,
    input::Input,
    renderer::Framebuffer,
    time::Time,
    GameContext,
    GameAction,
};

pub trait Game {
    type Context: GameContext;

    fn declare() -> Self
    where
        Self: Sized;

    fn state_data(
        &mut self,
    ) -> (
        &mut Vec<Box<dyn GameState<Self::Context>>>,
        &mut Self::Context,
    );

    fn start(&mut self) {}

    fn update(
        &mut self,
        time: &Time,
        input: &Input,
    ) -> GameAction {
        let action = {
            let (states, context) = self.state_data();

            match states.last_mut() {
                Some(state) => state.update(context, time, input),
                None => StateAction::None,
            }
        };

        self.handle_state_action(action)
    }

    fn render(
        &mut self,
        framebuffer: &mut Framebuffer,
    ) {
        let (states, context) = self.state_data();

        if states.is_empty() {
            return;
        }

        let mut first = states.len() - 1;

        while first > 0 && states[first].render_below() {
            first -= 1;
        }

        let mut canvas = Canvas::new(framebuffer);

        for state in &mut states[first..] {
            state.render(context, &mut canvas);
        }
    }

    fn handle_state_action(
        &mut self,
        action: StateAction<Self::Context>,
    ) -> GameAction {
        match action {
            StateAction::None => GameAction::None,

            StateAction::Pop => {
                self.pop_state();
                GameAction::None
            }

            StateAction::Push(state) => {
                self.push_boxed_state(state);
                GameAction::None
            }

            StateAction::Replace(state) => {
                self.replace_boxed_state(state);
                GameAction::None
            }

            StateAction::Exit => GameAction::Exit,
        }
    }

    fn push_state<S>(&mut self, state: S)
    where
        Self: Sized,
        S: GameState<Self::Context> + 'static,
    {
        self.push_boxed_state(Box::new(state));
    }

    fn push_boxed_state(
        &mut self,
        mut state: Box<dyn GameState<Self::Context>>,
    ) {
        let (states, context) = self.state_data();

        state.start(context);
        states.push(state);
    }

    fn pop_state(&mut self) {
        let (states, context) = self.state_data();

        if let Some(mut state) = states.pop() {
            state.end(context);
        }
    }

    fn replace_state<S>(&mut self, state: S)
    where
        Self: Sized,
        S: GameState<Self::Context> + 'static,
    {
        self.replace_boxed_state(Box::new(state));
    }

    fn replace_boxed_state(
        &mut self,
        mut state: Box<dyn GameState<Self::Context>>,
    ) {
        let (states, context) = self.state_data();

        if let Some(mut old_state) = states.pop() {
            old_state.end(context);
        }

        state.start(context);
        states.push(state);
    }
}