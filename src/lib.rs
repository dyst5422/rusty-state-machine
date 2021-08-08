use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ptr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeserializableTransitionRecord<'a, 'b, Context> {
    from_state_id: &'a str,
    to_state_id: &'a str,
    event_id: &'b str,
    edge_id: &'a str,
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeserializableEdge<'a, Info> {
    id: String,
    from_state_id: &'a str,
    to_state_id: &'a str,
    info: Info,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct State {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<EventPayload> {
    id: String,
    payload: EventPayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edge<'a, EdgeInfo> {
    id: String,
    from_state: &'a State,
    to_state: &'a State,
    info: EdgeInfo,
}

impl<'a, EdgeInfo> Edge<'a, EdgeInfo> {
    pub fn hydrate(
        deserializable_edge: DeserializableEdge<'a, EdgeInfo>,
        states: Vec<&'a State>,
    ) -> Edge<'a, EdgeInfo> {
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
pub struct TransitionRecord<'a, 'b, EventPayload, EdgeInfo, Context> {
    from_state: &'a State,
    to_state: &'a State,
    event: &'b Event<EventPayload>,
    edge: &'a Edge<'a, EdgeInfo>,
    context: Context,
}

impl<'a, 'b, EventPayload, EdgeInfo, Context>
TransitionRecord<'a, 'b, EventPayload, EdgeInfo, Context>
{
    pub fn hydrate(
        deserializable_transition_record: DeserializableTransitionRecord<'a, 'b, Context>,
        states: Vec<&'a State>,
        edges: Vec<&'a Edge<'a, EdgeInfo>>,
        events: Vec<&'b Event<EventPayload>>,
    ) -> TransitionRecord<'a, 'b, EventPayload, EdgeInfo, Context> {
        let from_state = states
            .iter()
            .find(|state| state.id == deserializable_transition_record.from_state_id)
            .expect(
                format!(
                    "Could not find a state with id: {}",
                    deserializable_transition_record.from_state_id
                )
                .as_str(),
            );
        let to_state = states
            .iter()
            .find(|state| state.id == deserializable_transition_record.to_state_id)
            .expect(
                format!(
                    "Could not find a state with id: {}",
                    deserializable_transition_record.to_state_id
                )
                .as_str(),
            );
        let event = events
            .iter()
            .find(|edge| edge.id == deserializable_transition_record.event_id)
            .expect(
                format!(
                    "Could not find an event with id: {}",
                    deserializable_transition_record.event_id
                )
                .as_str(),
            );
        let edge = edges
            .iter()
            .find(|edge| edge.id == deserializable_transition_record.edge_id)
            .expect(
                format!(
                    "Could not find an edge with id: {}",
                    deserializable_transition_record.edge_id
                )
                .as_str(),
            );
        TransitionRecord {
            from_state,
            to_state,
            event,
            edge,
            context: deserializable_transition_record.context,
        }
    }
}

impl<'a, 'b, EventPayload, EdgeInfo, Context>
    From<TransitionRecord<'a, 'b, EventPayload, EdgeInfo, Context>>
    for DeserializableTransitionRecord<'a, 'b, Context>
{
    fn from(state_transition: TransitionRecord<'a, 'b, EventPayload, EdgeInfo, Context>) -> Self {
        DeserializableTransitionRecord::<'a, 'b, Context> {
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

pub struct StateMachine<'a, 'b, EventPayload, EdgeInfo, Context> {
    pub transition_history: Vec<TransitionRecord<'a, 'b, EventPayload, EdgeInfo, Context>>,
    pub current_state: Option<&'a State>,
    pub current_context: Context,
    pub states: Vec<&'a State>,
    pub edges: Vec<&'a Edge<'a, EdgeInfo>>,
    state_to_edge_map: HashMap<&'a State, Vec<&'a Edge<'a, EdgeInfo>>>,
    event_handler: &'a EventHandler<EventPayload, EdgeInfo, Context>,
    start_dispatch_hook: Option<Box<dyn for<'c> FnMut(
        &'c Event<EventPayload>,
        &'c State,
        &'c Context,
        &'c Vec<&'a State>,
        &'c Vec<&'a Edge<'a, EdgeInfo>>
    ) + 'a>>,
    end_dispatch_hook: Option<Box<dyn for<'c> FnMut(
        &'c Event<EventPayload>,
        &'c State,
        &'c Context,
        &'c Vec<&'a State>,
        &'c Vec<&'a Edge<'a, EdgeInfo>>
    ) + 'a>>,
    // on_state_entry_hook: Option<Box<dyn for<'c> FnMut(
    //     &'c Event<EventPayload>,
    //     &'c State,
    //     &'c Edge<'a, EdgeInfo>,
    //     &'c Context,
    //     &'c Vec<&'a State>,
    //     &'c Vec<&'a Edge<'a, EdgeInfo>>
    // ) + 'a>>,
    // on_state_exit_hook: Option<Box<dyn for<'c> FnMut(
    //     &'c Event<EventPayload>,
    //     &'c State,
    //     &'c Edge<'a, EdgeInfo>,
    //     &'c Context,
    //     &'c Vec<&'a State>,
    //     &'c Vec<&'a Edge<'a, EdgeInfo>>
    // ) + 'a>>,
    on_edge_traversal_hook: Option<Box<dyn for<'c> FnMut(
        &'c Event<EventPayload>,
        &'c Edge<'a, EdgeInfo>,
        &'c Context,
        &'c Vec<&'a State>,
        &'c Vec<&'a Edge<'a, EdgeInfo>>
    ) + 'a>>,
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
        start_dispatch_hook: Option<impl for<'c> FnMut(
            &'c Event<EventPayload>,
            &'c State,
            &'c Context,
            &'c Vec<&'a State>,
            &'c Vec<&'a Edge<'a, EdgeInfo>>
        ) + 'a>,
        end_dispatch_hook: Option<impl for<'c> FnMut(
            &'c Event<EventPayload>,
            &'c State,
            &'c Context,
            &'c Vec<&'a State>,
            &'c Vec<&'a Edge<'a, EdgeInfo>>
        ) + 'a>,
        // on_state_entry_hook: Option<impl for<'c> FnMut(
        //     &'c Event<EventPayload>,
        //     &'c State,
        //     &'c Edge<'a, EdgeInfo>,
        //     &'c Context,
        //     &'c Vec<&'a State>,
        //     &'c Vec<&'a Edge<'a, EdgeInfo>>
        // ) + 'a>,
        // on_state_exit_hook: Option<impl for<'c> FnMut(
        //     &'c Event<EventPayload>,
        //     &'c State,
        //     &'c Edge<'a, EdgeInfo>,
        //     &'c Context,
        //     &'c Vec<&'a State>,
        //     &'c Vec<&'a Edge<'a, EdgeInfo>>
        // ) + 'a>,
        on_edge_traversal_hook: Option<impl for<'c> FnMut(
            &'c Event<EventPayload>,
            &'c Edge<'a, EdgeInfo>,
            &'c Context,
            &'c Vec<&'a State>,
            &'c Vec<&'a Edge<'a, EdgeInfo>>
        ) + 'a>,
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
            current_state: Some(initial_state),
            current_context: initial_context,
            states,
            edges,
            state_to_edge_map,
            event_handler,
            start_dispatch_hook: start_dispatch_hook.map(|h| Box::new(h) as Box<dyn for<'c> FnMut(
                &'c Event<EventPayload>,
                &'c State,
                &'c Context,
                &'c Vec<&'a State>,
                &'c Vec<&'a Edge<'a, EdgeInfo>>
            ) + 'a>),
            end_dispatch_hook: end_dispatch_hook.map(|h| Box::new(h) as Box<dyn for<'c> FnMut(
                &'c Event<EventPayload>,
                &'c State,
                &'c Context,
                &'c Vec<&'a State>,
                &'c Vec<&'a Edge<'a, EdgeInfo>>
            ) + 'a>),
            // on_state_entry_hook: on_state_entry_hook.map(|h| Box::new(h) as Box<dyn for<'c> FnMut(
            //     &'c Event<EventPayload>,
            //     &'c State,
            //     &'c Edge<'a, EdgeInfo>,
            //     &'c Context,
            //     &'c Vec<&'a State>,
            //     &'c Vec<&'a Edge<'a, EdgeInfo>>
            // ) + 'a>),
            // on_state_exit_hook: on_state_exit_hook.map(|h| Box::new(h) as Box<dyn for<'c> FnMut(
            //     &'c Event<EventPayload>,
            //     &'c State,
            //     &'c Edge<'a, EdgeInfo>,
            //     &'c Context,
            //     &'c Vec<&'a State>,
            //     &'c Vec<&'a Edge<'a, EdgeInfo>>
            // ) + 'a>),
            on_edge_traversal_hook: on_edge_traversal_hook.map(|h| Box::new(h) as Box<dyn for<'c> FnMut(
                &'c Event<EventPayload>,
                &'c Edge<'a, EdgeInfo>,
                &'c Context,
                &'c Vec<&'a State>,
                &'c Vec<&'a Edge<'a, EdgeInfo>>
            ) + 'a>),
        }
    }

    pub fn dispatch(&mut self, event: &'b Event<EventPayload>) {
        if let Some(start_dispatch_hook) = self.start_dispatch_hook.as_mut() {
            start_dispatch_hook(
                &event,
                self.current_state.unwrap(),
                &self.current_context,
                &self.states,
                &self.edges
            );
        }

        let current_state = self.current_state;
        let current_context = &self.current_context;
        let event_handler = self.event_handler;

        let mut transitioning_edge = None;
        let edges = self
            .state_to_edge_map
            .get(current_state.unwrap())
            .expect("Could not find a state");
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

        if let Some(end_dispatch_hook) = self.end_dispatch_hook.as_mut() {
            end_dispatch_hook(
                &event,
                self.current_state.unwrap(),
                &self.current_context,
                &self.states,
                &self.edges
            );
        }
    }

    fn transition(
        &mut self,
        event: &'b Event<EventPayload>,
        edge: &'a Edge<EdgeInfo>,
        context: Context,
    ) {
        // if let Some(on_state_exit_hook) = self.on_state_exit_hook.as_mut() {
        //     on_state_exit_hook(
        //         event,
        //         self.current_state.unwrap(),
        //         edge,
        //         &self.current_context,
        //         &self.states,
        //         &self.edges,
        //     );
        // }
        if let Some(on_edge_traversal_hook) = self.on_edge_traversal_hook.as_mut() {
            on_edge_traversal_hook(
                event,
                edge,
                &context,
                &self.states,
                &self.edges,
            );
        }
        self.transition_history.push(TransitionRecord {
            context: std::mem::replace(&mut self.current_context, context),
            edge,
            event,
            from_state: std::mem::replace(&mut self.current_state.unwrap(), edge.to_state),
            to_state: edge.to_state,
        });
        // if let Some(on_state_entry_hook) = self.on_state_entry_hook.as_mut() {
        //     on_state_entry_hook(
        //         event,
        //         self.current_state.unwrap(),
        //         edge,
        //         &self.current_context,
        //         &self.states,
        //         &self.edges,
        //     );
        // }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    // #[test]
    // fn it_appropriately_transitions() {
    //     let state1 = State {
    //         id: "first_state".to_string(),
    //     };
    //     let state2 = State {
    //         id: "second_state".to_string(),
    //     };
    //     let states = vec![&state1, &state2];
    //
    //     let edge1 = Edge {
    //         id: "from first to second".to_string(),
    //         from_state: &state1,
    //         to_state: &state2,
    //         info: "do it".to_string(),
    //     };
    //
    //     let edges = vec![&edge1];
    //
    //     fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
    //         Some(())
    //     }
    //
    //     let mut state_machine = StateMachine::new(
    //         &state1,
    //         (),
    //         states,
    //         edges,
    //         &(event_handler as EventHandler<(), String, ()>),
    //         None,
    //         None,
    //         None,
    //         None,
    //         None,
    //     );
    //
    //     let event1 = Event {
    //         id: "first event".to_string(),
    //         payload: (),
    //     };
    //
    //     state_machine.dispatch(&event1);
    //
    //     assert_eq!(state_machine.current_state, &state2);
    // }
    //
    // #[test]
    // fn it_appropriately_does_not_transition() {
    //     let state1 = State {
    //         id: "first_state".to_string(),
    //     };
    //     let state2 = State {
    //         id: "second_state".to_string(),
    //     };
    //     let states = vec![&state1, &state2];
    //
    //     let edge1 = Edge {
    //         id: "from first to second".to_string(),
    //         from_state: &state1,
    //         to_state: &state2,
    //         info: "do it".to_string(),
    //     };
    //
    //     let edges = vec![&edge1];
    //
    //     fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
    //         None
    //     }
    //
    //     let mut state_machine = StateMachine::new(
    //         &state1,
    //         (),
    //         states,
    //         edges,
    //         &(event_handler as EventHandler<(), String, ()>),
    //         None,
    //         None,
    //         None,
    //         None,
    //         None,
    //     );
    //
    //     let event1 = Event {
    //         id: "first event".to_string(),
    //         payload: (),
    //     };
    //
    //     state_machine.dispatch(&event1);
    //
    //     assert_eq!(state_machine.current_state, &state1);
    // }
    //
    // #[test]
    // #[should_panic(expected = "Cannot have multiple transitioning edges")]
    // fn it_panics_on_multiple_transitions() {
    //     let state1 = State {
    //         id: "first_state".to_string(),
    //     };
    //     let state2 = State {
    //         id: "second_state".to_string(),
    //     };
    //     let states = vec![&state1, &state2];
    //
    //     let edge1 = Edge {
    //         id: "from first to second".to_string(),
    //         from_state: &state1,
    //         to_state: &state2,
    //         info: "do it".to_string(),
    //     };
    //     let edge2 = Edge {
    //         id: "from first to second".to_string(),
    //         from_state: &state1,
    //         to_state: &state2,
    //         info: "do it again".to_string(),
    //     };
    //
    //     let edges = vec![&edge1, &edge2];
    //
    //     fn event_handler(_event: &Event<()>, _edge: &Edge<String>, _context: &()) -> Option<()> {
    //         Some(())
    //     }
    //
    //     let mut state_machine = StateMachine::new(
    //         &state1,
    //         (),
    //         states,
    //         edges,
    //         &(event_handler as EventHandler<(), String, ()>),
    //         None,
    //         None,
    //         None,
    //         None,
    //         None,
    //     );
    //
    //     let event1 = Event {
    //         id: "first event".to_string(),
    //         payload: (),
    //     };
    //
    //     state_machine.dispatch(&event1);
    // }

    #[test]
    fn it_calls_hooks<'a, 'c>() {
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

        let mut start_dispatch_hook_called = false;

        let start_dispatch_hook= |
            event: &Event<()>,
            current_state: &State,
            current_context: &(),
            states: &Vec<&State>,
            edges: &Vec<&Edge<String>>
        | {
            println!("start_dispatch_hook called");
            start_dispatch_hook_called = true;
        };

        let end_dispatch_hook= |
            event: &Event<()>,
            current_state: &State,
            current_context: &(),
            states: &Vec<&State>,
            edges: &Vec<&Edge<String>>
        | {
            for edge in edges {
                println!("{:?}", edge);
            }
            println!("{:?}", event);
        };

        let enter_state_hook = |
            event: &Event<()>,
            edge: &Edge<String>,
            current_state: &State,
            current_context: &(),
            states: &Vec<&State>,
            edges: &Vec<&Edge<String>>
        | {
            for edge in edges {
                println!("{:?}", edge);
            }
            println!("{:?}", event);
        };

        // let exit_state_hook = |
        //     event: &Event<()>,
        //     edge: &Edge<String>,
        //     current_state: &State,
        //     current_context: &(),
        //     states: &Vec<&State>,
        //     edges: &Vec<&Edge<String>>
        // | {
        //     for edge in edges {
        //         println!("{:?}", edge);
        //     }
        //     println!("{:?}", event);
        // };
        //
        let traverse_edge_hook = |
            event: &Event<()>,
            edge: &Edge<String>,
            current_context: &(),
            states: &Vec<&State>,
            edges: &Vec<&Edge<String>>
        | {
            for edge in edges {
                println!("{:?}", edge);
            }
            println!("{:?}", event);
        };

        let mut state_machine: StateMachine<(), String, ()> = StateMachine::new(
            &state1,
            (),
            states,
            edges,
            &(event_handler as EventHandler<(), String, ()>),
            Some(start_dispatch_hook),
            Some(end_dispatch_hook),
            Some(traverse_edge_hook),
            // None,
            // None,
        );

        let event1 = Event {
            id: "first event".to_string(),
            payload: (),
        };

        state_machine.dispatch(&event1);
        std::mem::drop(state_machine);
        println!("start_dispatch_hook_called: {}", &start_dispatch_hook_called);
        assert_eq!(start_dispatch_hook_called, true);
        // assert_eq!(&state_machine.current_state.unwrap(), &state2);

    }
}
