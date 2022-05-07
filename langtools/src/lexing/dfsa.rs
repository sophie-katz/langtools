use super::dfsa_error::{DFSAError, Result};
use super::dfsa_types::DFSAId;
use std::collections::HashMap;
use std::hash::Hash;
use std::option::Option;
use std::vec::Vec;

#[derive(Debug)]
struct DFSAState<TElement: Eq + Hash, TAction> {
    action: Option<TAction>,
    transitions: HashMap<TElement, DFSAId>,
}

#[derive(Debug)]
pub struct DFSA<TElement: Eq + Hash, TAction> {
    states: Vec<DFSAState<TElement, TAction>>,
    start_id: Option<DFSAId>,
}

impl<TElement: Eq + Hash, TAction> DFSAState<TElement, TAction> {
    pub fn new() -> Self {
        Self {
            action: None,
            transitions: HashMap::new(),
        }
    }

    pub fn new_with_action(action: TAction) -> Self {
        Self {
            action: Some(action),
            transitions: HashMap::new(),
        }
    }
}

impl<TElement: Eq + Hash, TAction> DFSA<TElement, TAction> {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            start_id: None,
        }
    }

    #[must_use]
    pub fn add_state(&mut self) -> DFSAId {
        let id = self.states.len();
        self.states.push(DFSAState::new());
        id
    }

    #[must_use]
    pub fn add_state_with_action(&mut self, action: TAction) -> DFSAId {
        let id = self.states.len();
        self.states.push(DFSAState::new_with_action(action));
        id
    }

    pub fn add_transition(
        &mut self,
        from_id: DFSAId,
        on_element: TElement,
        to_id: DFSAId,
    ) -> Result<()> {
        if !self.is_id_in_bounds(to_id) {
            return Err(DFSAError::OutOfRangeId(to_id));
        }

        match self
            .try_get_state_mut(from_id)?
            .transitions
            .insert(on_element, to_id)
        {
            Some(_) => Err(DFSAError::TransitionAlreadyExists),
            None => Ok(()),
        }
    }

    pub fn try_get_start_id(&self) -> Result<DFSAId> {
        if let Some(result) = self.start_id {
            Ok(result)
        } else {
            Err(DFSAError::NoStartId)
        }
    }

    pub fn set_start_id(&mut self, id: DFSAId) -> Result<()> {
        if self.is_id_in_bounds(id) {
            self.start_id = Some(id);
            Ok(())
        } else {
            Err(DFSAError::OutOfRangeId(id))
        }
    }

    pub fn try_get_state_action(&self, id: DFSAId) -> Result<&TAction> {
        self.try_get_state(id)?
            .action
            .as_ref()
            .ok_or(DFSAError::StateHasNoAction(id))
    }

    pub fn set_state_action(&mut self, id: DFSAId, action: Option<TAction>) -> Result<()> {
        self.try_get_state_mut(id)?.action = action;
        Ok(())
    }

    pub fn try_get_transition(&self, from_id: DFSAId, on_element: TElement) -> Result<DFSAId> {
        self.try_get_state(from_id)?
            .transitions
            .get(&on_element)
            .ok_or(DFSAError::NoSuchTransition)
            .copied()
    }

    fn is_id_in_bounds(&self, id: DFSAId) -> bool {
        id < self.states.len()
    }

    fn try_get_state(&self, id: DFSAId) -> Result<&DFSAState<TElement, TAction>> {
        self.states.get(id).ok_or(DFSAError::OutOfRangeId(id))
    }

    fn try_get_state_mut(&mut self, id: DFSAId) -> Result<&mut DFSAState<TElement, TAction>> {
        self.states.get_mut(id).ok_or(DFSAError::OutOfRangeId(id))
    }
}

impl<T: Eq + Hash, U> Default for DFSA<T, U> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfsa_add_transition_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        dfsa.add_transition(start, 'a', a)?;
        Ok(())
    }

    #[test]
    fn test_dfsa_add_transition_bad_from_id() {
        let mut dfsa = DFSA::<char, String>::new();
        let a = dfsa.add_state();

        assert_eq!(
            dfsa.add_transition(100, 'a', a),
            Err(DFSAError::OutOfRangeId(100))
        );
    }

    #[test]
    fn test_dfsa_add_transition_bad_to_id() {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();

        assert_eq!(
            dfsa.add_transition(start, 'a', 100),
            Err(DFSAError::OutOfRangeId(100))
        );
    }

    #[test]
    fn test_dfsa_add_transition_duplicate() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        dfsa.add_transition(start, 'a', a)?;

        assert_eq!(
            dfsa.add_transition(start, 'a', a),
            Err(DFSAError::TransitionAlreadyExists)
        );

        Ok(())
    }

    #[test]
    fn test_dfsa_try_get_transition_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        dfsa.add_transition(start, 'a', a)?;

        assert_eq!(dfsa.try_get_transition(start, 'a'), Ok(a));

        Ok(())
    }

    #[test]
    fn test_dfsa_try_get_transition_bad_element() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        dfsa.add_transition(start, 'a', a)?;

        assert_eq!(
            dfsa.try_get_transition(start, 'b'),
            Err(DFSAError::NoSuchTransition)
        );

        Ok(())
    }

    #[test]
    fn test_dfsa_try_get_transition_bad_id() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        dfsa.add_transition(start, 'a', a)?;

        assert_eq!(
            dfsa.try_get_transition(100, 'b'),
            Err(DFSAError::OutOfRangeId(100))
        );

        Ok(())
    }

    #[test]
    fn test_dfsa_set_start_id_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();

        dfsa.set_start_id(start)?;
        dfsa.set_start_id(start)?;

        Ok(())
    }

    #[test]
    fn test_dfsa_set_start_id_bad() {
        let mut dfsa = DFSA::<char, String>::new();

        assert_eq!(dfsa.set_start_id(100), Err(DFSAError::OutOfRangeId(100)));
    }

    #[test]
    fn test_dfsa_try_get_start_id_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        dfsa.set_start_id(start)?;

        dfsa.try_get_start_id()?;

        Ok(())
    }

    #[test]
    fn test_dfsa_try_get_start_id_bad() {
        let mut dfsa = DFSA::<char, String>::new();
        let _ = dfsa.add_state();

        assert_eq!(dfsa.try_get_start_id(), Err(DFSAError::NoStartId));
    }

    #[test]
    fn test_dfsa_set_state_action_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();

        dfsa.set_state_action(start, None)?;
        dfsa.set_state_action(start, Some(String::from("hi")))?;

        Ok(())
    }

    #[test]
    fn test_dfsa_set_state_action_bad() {
        let mut dfsa = DFSA::<char, String>::new();

        assert_eq!(
            dfsa.set_state_action(100, None),
            Err(DFSAError::OutOfRangeId(100))
        );
    }
}
