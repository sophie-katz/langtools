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

use std::collections::HashMap;

use super::message::{Message, Severity};

#[readonly::make]
pub struct MessageContext {
    pub messages: Vec<Message>,
    severity_counts: HashMap<Severity, usize>,
}

impl MessageContext {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            severity_counts: HashMap::new(),
        }
    }

    pub fn emit(&mut self, message: Message) {
        match self.severity_counts.get_mut(&message.severity) {
            Some(count) => {
                *count += 1usize;
            }
            None => {
                self.severity_counts.insert(message.severity, 1);
            }
        }

        self.messages.push(message);
    }

    pub fn count_with_severity(&self, severity: Severity) -> usize {
        self.severity_counts
            .get(&severity)
            .copied()
            .unwrap_or(0usize)
    }
}

impl Default for MessageContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_context_simple() {
        let mut message_context = MessageContext::new();
        message_context.emit(Message::new_global(Severity::Warning, String::from("a")));
        message_context.emit(Message::new_global(Severity::Warning, String::from("b")));
        message_context.emit(Message::new_global(Severity::Info, String::from("c")));

        assert_eq!(message_context.count_with_severity(Severity::Warning), 2);
        assert_eq!(message_context.count_with_severity(Severity::Info), 1);
        assert_eq!(message_context.count_with_severity(Severity::Error), 0);
    }
}
