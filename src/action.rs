use crate::task::Task;

#[derive(Debug)]
pub struct Action<I, Message> {
    pub instruction: Option<I>,
    pub task: Task<Message>,
}

#[allow(dead_code)]
impl<I, Message> Action<I, Message> {
    pub fn none() -> Self {
        Self {
            instruction: None,
            task: Task::none(),
        }
    }

    pub fn instruction(instruction: I) -> Self {
        Self {
            instruction: Some(instruction),
            task: Task::none(),
        }
    }

    pub fn task(task: Task<Message>) -> Self {
        Self {
            instruction: None,
            task,
        }
    }

    pub fn map<N>(self, f: impl Fn(Message) -> N + 'static) -> Action<I, N> {
        Action {
            instruction: self.instruction,
            task: self.task.map(f),
        }
    }

    pub fn map_instruction<N>(self, f: impl Fn(I) -> N + 'static) -> Action<N, Message> {
        Action {
            instruction: self.instruction.map(f),
            task: self.task,
        }
    }

    pub fn with_instruction(mut self, instruction: I) -> Self {
        self.instruction = Some(instruction);
        self
    }

    pub fn with_task(mut self, task: Task<Message>) -> Self {
        self.task = task;
        self
    }
}
