pub struct Task<M> {
    messages: Option<Vec<M>>,
}

impl<M> Task<M> {
    pub fn none() -> Self {
        Self { messages: None }
    }

    pub fn new(message: M) -> Self {
        Self {
            messages: Some(vec![message]),
        }
    }

    pub fn batch<I>(tasks: I) -> Self
    where
        I: IntoIterator<Item = Task<M>>,
    {
        let tasks: Vec<M> = tasks
            .into_iter()
            .map(|t| t.messages)
            .filter_map(|opt| opt)
            .flatten()
            .collect();

        Self {
            messages: Some(tasks),
        }
    }

    pub fn map<N>(self, f: impl Fn(M) -> N) -> Task<N> {
        Task {
            messages: self
                .messages
                .and_then(|v| Some(v.into_iter().map(|m| f(m)).collect())),
        }
    }

    pub fn is_none(&self) -> bool {
        self.messages.is_none()
    }

    pub fn into_inner(self) -> Vec<M> {
        self.messages.unwrap_or(vec![])
    }
}
