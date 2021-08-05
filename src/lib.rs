use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ptr;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct State {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<Payload> {
    id: String,
    payload: Payload,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edge<'a, Info> {
    id: String,
    from_state: &'a State,
    to_state: &'a State,
    info: Info,
}

impl<'a, Info> Edge<'a, Info> {
    pub fn hydrate(
        deserializable_edge: DeserializableEdge<'a, Info>,
        states: Vec<&'a State>,
    ) -> Edge<'a, Info> {
        let from_state = states
            .iter()
            .find(|state| state.id == deserializable_edge.from_state_id)
            .expect(
                format!(
                    "Could not find a state with id: {}",
                    deserializable_edge.from_state_id
                )
                .as_str(),
            );
        let to_state = states
            .iter()
            .find(|state| state.id == deserializable_edge.to_state_id)
            .expect(
                format!(
                    "Could not find a state with id: {}",
                    deserializable_edge.to_state_id
                )
                .as_str(),
            );
        Edge {
            id: deserializable_edge.id,
            from_state,
            to_state,
            info: deserializable_edge.info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeserializableEdge<'a, Info> {
    id: String,
    from_state_id: &'a str,
    to_state_id: &'a str,
    info: Info,
}

impl<'a, Info> From<Edge<'a, Info>> for DeserializableEdge<'a, Info> {
    fn from(edge: Edge<'a, Info>) -> Self {
        DeserializableEdge {
            id: edge.id,
            from_state_id: &edge.from_state.id,
            to_state_id: &edge.to_state.id,
            info: edge.info,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StateTransition<'a, 'b, EventPayload, EdgeInfo, Context> {
    from_state: &'a State,
    to_state: &'a State,
    event: &'b Event<EventPayload>,
    edge: &'a Edge<'a, EdgeInfo>,
    context: Context,
}

impl<'a, 'b, EventPayload, EdgeInfo, Context>
    StateTransition<'a, 'b, EventPayload, EdgeInfo, Context>
{
    pub fn hydrate(
        deserializable_state_transition: DeserializableStateTransition<'a, 'b, Context>,
        states: Vec<&'a State>,
        edges: Vec<&'a Edge<'a, EdgeInfo>>,
        events: Vec<&'b Event<EventPayload>>,
    ) -> StateTransition<'a, 'b, EventPayload, EdgeInfo, Context> {
        let from_state = states
            .iter()
            .find(|state| state.id == deserializable_state_transition.from_state_id)
            .expect(
                format!(
                    "Could not find a state with id: {}",
                    deserializable_state_transition.from_state_id
                )
                .as_str(),
            );
        let to_state = states
            .iter()
            .find(|state| state.id == deserializable_state_transition.to_state_id)
            .expect(
                format!(
                    "Could not find a state with id: {}",
                    deserializable_state_transition.to_state_id
                )
                .as_str(),
            );
        let event = events
            .iter()
            .find(|edge| edge.id == deserializable_state_transition.event_id)
            .expect(
                format!(
                    "Could not find an event with id: {}",
                    deserializable_state_transition.event_id
                )
                .as_str(),
            );
        let edge = edges
            .iter()
            .find(|edge| edge.id == deserializable_state_transition.edge_id)
            .expect(
                format!(
                    "Could not find an edge with id: {}",
                    deserializable_state_transition.edge_id
                )
                .as_str(),
            );
        StateTransition {
            from_state,
            to_state,
            event,
            edge,
            context: deserializable_state_transition.context,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeserializableStateTransition<'a, 'b, Context> {
    from_state_id: &'a str,
    to_state_id: &'a str,
    event_id: &'b str,
    edge_id: &'a str,
    context: Context,
}

impl<'a, 'b, EventPayload, EdgeInfo, Context>
    From<StateTransition<'a, 'b, EventPayload, EdgeInfo, Context>>
    for DeserializableStateTransition<'a, 'b, Context>
{
    fn from(state_transition: StateTransition<'a, 'b, EventPayload, EdgeInfo, Context>) -> Self {
        DeserializableStateTransition::<'a, 'b, Context> {
            from_state_id: &state_transition.from_state.id,
            to_state_id: &state_transition.to_state.id,
            event_id: &state_transition.event.id,
            edge_id: &state_transition.edge.id,
            context: state_transition.context,
        }
    }
}

pub type EventHandler<EventPayload, EdgeInfo, Context> =
    fn(&Event<EventPayload>, &Edge<EdgeInfo>, &Context) -> Option<Context>;
pub type DispatchHook<'a, 'b, EventPayload, EdgeInfo, Context> =
    dyn FnMut(&mut StateMachine<'a, 'b, EventPayload, EdgeInfo, Context>, &Event<EventPayload>);
pub type TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context> = fn(
    &mut StateMachine<'a, 'b, EventPayload, EdgeInfo, Context>,
    &Edge<'a, EdgeInfo>,
    &Event<EventPayload>,
);

pub struct StateMachine<'a, 'b, EventPayload, EdgeInfo, Context> {
    pub transition_history: Vec<StateTransition<'a, 'b, EventPayload, EdgeInfo, Context>>,
    pub current_state: &'a State,
    pub current_context: Context,
    pub states: Vec<&'a State>,
    pub edges: Vec<&'a Edge<'a, EdgeInfo>>,
    pub event_handler: &'a EventHandler<EventPayload, EdgeInfo, Context>,
    pub start_dispatch_hook: Option<&'a mut DispatchHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
    pub end_dispatch_hook: Option<&'a DispatchHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
    pub on_state_entry_hook: Option<&'a TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
    pub on_state_exit_hook: Option<&'a TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
    pub on_edge_traversal_hook: Option<&'a TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
    state_to_edge_map: HashMap<&'a State, Vec<&'a Edge<'a, EdgeInfo>>>,
}

impl<'a, 'b, EventPayload, EdgeInfo, Context> Debug
    for StateMachine<'a, 'b, EventPayload, EdgeInfo, Context>
where
    EventPayload: Debug,
    EdgeInfo: Debug,
    Context: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("current_state", &self.current_state)
            .field("transition_history", &self.transition_history)
            .finish()
    }
}

impl<'a, 'b, EventPayload, EdgeInfo, Context> StateMachine<'a, 'b, EventPayload, EdgeInfo, Context>
where
    EventPayload: Debug,
    EdgeInfo: Debug,
    Context: Debug,
{
    pub fn new(
        initial_state: &'a State,
        initial_context: Context,
        states: Vec<&'a State>,
        edges: Vec<&'a Edge<'a, EdgeInfo>>,
        event_handler: &'a EventHandler<EventPayload, EdgeInfo, Context>,
        start_dispatch_hook: Option<&'a mut DispatchHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
        end_dispatch_hook: Option<&'a DispatchHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
        on_state_entry_hook: Option<&'a TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
        on_state_exit_hook: Option<&'a TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
        on_edge_traversal_hook: Option<&'a TransitionHook<'a, 'b, EventPayload, EdgeInfo, Context>>,
    ) -> StateMachine<'a, 'b, EventPayload, EdgeInfo, Context> {
        let mut state_to_edge_map = HashMap::new();
        for state in states.clone() {
            let mut state_edges = Vec::new();
            for edge in &edges {
                if ptr::eq(edge.from_state, state) {
                    state_edges.push(edge.clone());
                }
            }
            state_to_edge_map.insert(state, state_edges);
        }
        StateMachine {
            transition_history: Vec::new(),
            current_state: initial_state,
            current_context: initial_context,
            states,
            edges,
            state_to_edge_map,
            event_handler,
            start_dispatch_hook,
            end_dispatch_hook,
            on_state_entry_hook,
            on_state_exit_hook,
            on_edge_traversal_hook,
        }
    }

    pub fn dispatch(&mut self, event: &'b Event<EventPayload>) {
        if self.start_dispatch_hook.is_some() {
            self.start_dispatch_hook.unwrap()(self, event);
        }
        // if let Some(start_dispatch_hook) = self.start_dispatch_hook {
        //     start_dispatch_hook(self, event);
        // }

        let current_state = self.current_state;
        let current_context = &self.current_context;
        let event_handler = self.event_handler;

        let mut transitioning_edge = None;
        let edges = self.state_to_edge_map.get(current_state).expect("Could not find a state");
        for edge in edges {
            let event_handler_result = event_handler(event, edge, current_context);
            if let Some(new_context) = event_handler_result {
                assert!(
                    transitioning_edge.is_none(),
                    "Cannot have multiple transitioning edges"
                );
                transitioning_edge = Some((edge, new_context));
            }
        }
        if let Some((edge, new_context)) = transitioning_edge {
            self.transition(event, edge, new_context);
        }

        if let Some(end_dispatch_hook) = self.end_dispatch_hook {
            end_dispatch_hook(self, event);
        }
    }

    fn transition(
        &mut self,
        event: &'b Event<EventPayload>,
        edge: &'a Edge<EdgeInfo>,
        context: Context,
    ) {
        if let Some(on_state_exit_hook) = self.on_state_exit_hook {
            on_state_exit_hook(self, edge, event);
        }
        if let Some(on_edge_traversal_hook) = self.on_edge_traversal_hook {
            on_edge_traversal_hook(self, edge, event);
        }
        self.transition_history.push(StateTransition {
            context: std::mem::replace(&mut self.current_context, context),
            edge,
            event,
            from_state: std::mem::replace(&mut self.current_state, edge.to_state),
            to_state: edge.to_state,
        });
        if let Some(on_state_entry_hook) = self.on_state_entry_hook {
            on_state_entry_hook(self, edge, event);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_appropriately_transitions() {
        let state1 = State {
            id: "first_state".to_string(),
        };
        let state2 = State {
            id: "second_state".to_string(),
        };
        let states = vec![&state1, &state2];

        let edge1 = Edge {
            id: "from first to second".to_string(),
            from_state: &state1,
            to_state: &state2,
            info: "do it".to_string(),
        };

        let edges = vec![&edge1];

        fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
            Some(())
        }

        let mut state_machine = StateMachine::new(
            &state1,
            (),
            states,
            edges,
            &(event_handler as EventHandler<(), String, ()>),
            None,
            None,
            None,
            None,
            None,
        );

        let event1 = Event {
            id: "first event".to_string(),
            payload: (),
        };

        state_machine.dispatch(&event1);

        assert_eq!(state_machine.current_state, &state2);
    }

    #[test]
    fn it_appropriately_does_not_transition() {
        let state1 = State {
            id: "first_state".to_string(),
        };
        let state2 = State {
            id: "second_state".to_string(),
        };
        let states = vec![&state1, &state2];

        let edge1 = Edge {
            id: "from first to second".to_string(),
            from_state: &state1,
            to_state: &state2,
            info: "do it".to_string(),
        };

        let edges = vec![&edge1];

        fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
            None
        }

        let mut state_machine = StateMachine::new(
            &state1,
            (),
            states,
            edges,
            &(event_handler as EventHandler<(), String, ()>),
            None,
            None,
            None,
            None,
            None,
        );

        let event1 = Event {
            id: "first event".to_string(),
            payload: (),
        };

        state_machine.dispatch(&event1);

        assert_eq!(state_machine.current_state, &state1);
    }

    #[test]
    #[should_panic(expected = "Cannot have multiple transitioning edges")]
    fn it_panics_on_multiple_transitions() {
        let state1 = State {
            id: "first_state".to_string(),
        };
        let state2 = State {
            id: "second_state".to_string(),
        };
        let states = vec![&state1, &state2];

        let edge1 = Edge {
            id: "from first to second".to_string(),
            from_state: &state1,
            to_state: &state2,
            info: "do it".to_string(),
        };
        let edge2 = Edge {
            id: "from first to second".to_string(),
            from_state: &state1,
            to_state: &state2,
            info: "do it again".to_string(),
        };

        let edges = vec![&edge1, &edge2];

        fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
            Some(())
        }

        let mut state_machine = StateMachine::new(
            &state1,
            (),
            states,
            edges,
            &(event_handler as EventHandler<(), String, ()>),
            None,
            None,
            None,
            None,
            None,
        );

        let event1 = Event {
            id: "first event".to_string(),
            payload: (),
        };

        state_machine.dispatch(&event1);
    }

    #[test]
    fn it_calls_hooks() {
        let state1 = State {
            id: "first_state".to_string(),
        };
        let state2 = State {
            id: "second_state".to_string(),
        };
        let states = vec![&state1, &state2];

        let edge1 = Edge {
            id: "from first to second".to_string(),
            from_state: &state1,
            to_state: &state2,
            info: "do it".to_string(),
        };

        let edges = vec![&edge1];

        fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
            Some(())
        }

        let mut state_dispatch_hook_called = false;
        let mut state_dispatch_hook = |state_machine: &mut StateMachine<(), String, ()>, event: &Event<()>| state_dispatch_hook_called = true;

        let mut state_machine = StateMachine::new(
            &state1,
            (),
            states,
            edges,
            &(event_handler as EventHandler<(), String, ()>),
            Some(&mut state_dispatch_hook),
            None,
            None,
            None,
            None,
        );

        let event1 = Event {
            id: "first event".to_string(),
            payload: (),
        };

        state_machine.dispatch(&event1);

        assert_eq!(state_machine.current_state, &state2);
    }
}
