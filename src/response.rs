/// Response.
#[derive(Debug, Clone, Copy)]
pub struct Response<T> {
    data: T,
}

impl<T> Response<T> {
    /// Create a new response.
    pub fn new(data: T) -> Self {
        Self { data }
    }

    /// Convert into inner data.
    pub fn into_inner(self) -> T {
        self.data
    }
}
