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

use super::dfsa::DFSA;
use super::fsa_error::Result;
use super::fsa_types::FSAId;
use std::hash::Hash;

#[derive(Debug)]
pub struct DFSAExecutor<'dfsa, TElement: Eq + Hash, TAction> {
    dfsa: &'dfsa DFSA<TElement, TAction>,
    start_id: FSAId,
    current_id: FSAId,
}

impl<'dfsa, TElement: Eq + Hash, TAction> DFSAExecutor<'dfsa, TElement, TAction> {
    pub fn new(dfsa: &'dfsa DFSA<TElement, TAction>) -> Result<Self> {
        let start_id = dfsa.try_get_start_id()?;

        Ok(Self {
            dfsa,
            start_id,
            current_id: start_id,
        })
    }

    pub fn reset(&mut self) {
        self.current_id = self.start_id
    }

    pub fn step(&mut self, element: TElement) -> Result<()> {
        match self.dfsa.try_get_transition(self.current_id, element) {
            Ok(next_id) => {
                self.current_id = next_id;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn current_action(&self) -> Option<&'dfsa TAction> {
        self.dfsa.try_get_state_action(self.current_id).ok()
    }

    pub fn is_at_start(&self) -> bool {
        self.start_id == self.current_id
    }
}

#[cfg(test)]
mod tests {
    use super::super::fsa_error::FSAError;
    use super::*;

    #[test]
    fn test_dfsa_executor_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        let ab = dfsa.add_state_with_action(String::from("ab"));
        let b = dfsa.add_state_with_action(String::from("b"));
        dfsa.add_transition(start, 'a', a)?;
        dfsa.add_transition(a, 'b', ab)?;
        dfsa.add_transition(start, 'b', b)?;
        dfsa.set_start_id(start)?;

        let mut dfsa_executor = DFSAExecutor::new(&dfsa)?;

        assert_eq!(dfsa_executor.step('a'), Ok(()));
        assert_eq!(dfsa_executor.current_action(), None);
        assert_eq!(dfsa_executor.step('b'), Ok(()));
        assert_eq!(
            dfsa_executor.current_action().cloned(),
            Some(String::from("ab"))
        );
        assert_eq!(dfsa_executor.step('a'), Err(FSAError::NoSuchTransition));

        dfsa_executor.reset();

        assert_eq!(dfsa_executor.step('b'), Ok(()));
        assert_eq!(
            dfsa_executor.current_action().cloned(),
            Some(String::from("b"))
        );
        assert_eq!(dfsa_executor.step('a'), Err(FSAError::NoSuchTransition));

        Ok(())
    }

    #[test]
    fn test_dfsa_executor_new_no_start_id() {
        let dfsa = DFSA::<char, String>::new();

        assert_eq!(
            DFSAExecutor::new(&dfsa).map(|_| ()),
            Err(FSAError::NoStartId)
        );
    }
}
