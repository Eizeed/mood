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
            .filter_map(|t| t.messages)
            .flatten()
            .collect();

        Self {
            messages: Some(tasks),
        }
    }

    pub fn map<N>(self, f: impl Fn(M) -> N) -> Task<N> {
        Task {
            messages: self.messages.map(|v| v.into_iter().map(f).collect()),
        }
    }

    pub fn is_none(&self) -> bool {
        self.messages.is_none()
    }

    pub fn into_inner(self) -> Vec<M> {
        self.messages.unwrap_or_default()
    }
}
