use crate::task::Task;

pub struct Action<I, Message> {
    pub instruction: Option<I>,
    pub message: Task<Message>,
}

impl<I, Message> Action<I, Message> {
    pub fn none() -> Self {
        Self {
            instruction: None,
            message: Task::none(),
        }
    }

    pub fn instruction(instruction: I) -> Self {
        Self {
            instruction: Some(instruction),
            message: Task::none(),
        }
    }

    pub fn message(message: Message) -> Self {
        Self {
            instruction: None,
            message: Task::new(message),
        }
    }

    pub fn map<N>(self, f: impl Fn(Message) -> N + 'static) -> Action<I, N> {
        Action {
            instruction: self.instruction,
            message: self.message.map(f),
        }
    }

    pub fn map_instruction<N>(self, f: impl Fn(I) -> N + 'static) -> Action<N, Message> {
        Action {
            instruction: self.instruction.map(f),
            message: self.message,
        }
    }

    pub fn with_instruction(mut self, instruction: I) -> Self {
        self.instruction = Some(instruction);
        self
    }

    pub fn with_message(mut self, message: Message) -> Self {
        self.message = Task::new(message);
        self
    }
}
