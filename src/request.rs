/// Request.
#[derive(Debug, Clone, Copy)]
pub struct Request<T> {
    data: T,
}

impl<T> Request<T> {
    /// Create a new request.
    pub fn new(data: T) -> Self {
        Self { data }
    }

    /// Convert into inner data.
    pub fn into_inner(self) -> T {
        self.data
    }
}
