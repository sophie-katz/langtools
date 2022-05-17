// MIT License
//
// Copyright (c) 2022 Sophie Katz
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::fsa_error::{FSAError, Result};
use super::fsa_types::FSAId;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::option::Option;
use std::vec::Vec;

#[derive(Debug)]
struct NFSAState<TElement: Eq + Hash, TAction> {
    action: Option<TAction>,
    transitions_value: HashMap<TElement, FSAId>,
    transitions_epsilon: HashSet<FSAId>,
}

#[derive(Debug)]
pub struct NFSA<TElement: Eq + Hash, TAction> {
    states: Vec<NFSAState<TElement, TAction>>,
    start_id: Option<FSAId>,
}

impl<TElement: Eq + Hash, TAction> NFSAState<TElement, TAction> {
    pub fn new() -> Self {
        Self {
            action: None,
            transitions_value: HashMap::new(),
            transitions_epsilon: HashSet::new(),
        }
    }

    pub fn new_with_action(action: TAction) -> Self {
        Self {
            action: Some(action),
            transitions_value: HashMap::new(),
            transitions_epsilon: HashSet::new(),
        }
    }
}

impl<TElement: Eq + Hash, TAction> NFSA<TElement, TAction> {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            start_id: None,
        }
    }

    #[must_use]
    pub fn add_state(&mut self) -> FSAId {
        let id = self.states.len();
        self.states.push(NFSAState::new());
        id
    }

    #[must_use]
    pub fn add_state_with_action(&mut self, action: TAction) -> FSAId {
        let id = self.states.len();
        self.states.push(NFSAState::new_with_action(action));
        id
    }

    pub fn add_transition_value(
        &mut self,
        from_id: FSAId,
        on_element: TElement,
        to_id: FSAId,
    ) -> Result<()> {
        if !self.is_id_in_bounds(to_id) {
            return Err(FSAError::OutOfRangeId(to_id));
        }

        match self
            .try_get_state_mut(from_id)?
            .transitions
            .insert(on_element, to_id)
        {
            Some(_) => Err(FSAError::TransitionAlreadyExists),
            None => Ok(()),
        }
    }

    pub fn add_transition_epsilon(&mut self, from_id: FSAId, to_id: FSAId) -> Result<()> {
        if !self.is_id_in_bounds(to_id) {
            return Err(FSAError::OutOfRangeId(to_id));
        }

        match self
            .try_get_state_mut(from_id)?
            .transitions
            .insert(on_element, to_id)
        {
            Some(_) => Err(FSAError::TransitionAlreadyExists),
            None => Ok(()),
        }
    }

    pub fn try_get_start_id(&self) -> Result<FSAId> {
        if let Some(result) = self.start_id {
            Ok(result)
        } else {
            Err(FSAError::NoStartId)
        }
    }

    pub fn set_start_id(&mut self, id: FSAId) -> Result<()> {
        if self.is_id_in_bounds(id) {
            self.start_id = Some(id);
            Ok(())
        } else {
            Err(FSAError::OutOfRangeId(id))
        }
    }

    pub fn try_get_state_action(&self, id: FSAId) -> Result<&TAction> {
        self.try_get_state(id)?
            .action
            .as_ref()
            .ok_or(FSAError::StateHasNoAction(id))
    }

    pub fn set_state_action(&mut self, id: FSAId, action: Option<TAction>) -> Result<()> {
        self.try_get_state_mut(id)?.action = action;
        Ok(())
    }

    pub fn try_get_transition(&self, from_id: FSAId, on_element: TElement) -> Result<FSAId> {
        self.try_get_state(from_id)?
            .transitions
            .get(&on_element)
            .ok_or(FSAError::NoSuchTransition)
            .copied()
    }

    fn is_id_in_bounds(&self, id: FSAId) -> bool {
        id < self.states.len()
    }

    fn try_get_state(&self, id: FSAId) -> Result<&DFSAState<TElement, TAction>> {
        self.states.get(id).ok_or(FSAError::OutOfRangeId(id))
    }

    fn try_get_state_mut(&mut self, id: FSAId) -> Result<&mut DFSAState<TElement, TAction>> {
        self.states.get_mut(id).ok_or(FSAError::OutOfRangeId(id))
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
            Err(FSAError::OutOfRangeId(100))
        );
    }

    #[test]
    fn test_dfsa_add_transition_bad_to_id() {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();

        assert_eq!(
            dfsa.add_transition(start, 'a', 100),
            Err(FSAError::OutOfRangeId(100))
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
            Err(FSAError::TransitionAlreadyExists)
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
            Err(FSAError::NoSuchTransition)
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
            Err(FSAError::OutOfRangeId(100))
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

        assert_eq!(dfsa.set_start_id(100), Err(FSAError::OutOfRangeId(100)));
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

        assert_eq!(dfsa.try_get_start_id(), Err(FSAError::NoStartId));
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
            Err(FSAError::OutOfRangeId(100))
        );
    }
}
