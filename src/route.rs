use http::{Request};

pub struct RoutedRequest<B> {
    request: Request<B>,
    split: usize, // pre-routed path split
}

impl<B> RoutedRequest<B> {
    pub fn inner(self) -> Request<B> {
        self.request
    }
    
    pub fn route(self, path: &str) -> Self {
        Self { split: self.request.uri().path().len() - path.len(), ..self }
    }

    pub fn prefix(&self) -> &str {
        self.request.uri().path().split_at(self.split).0
    }
    
    pub fn path(&self) -> &str {
        self.request.uri().path().split_at(self.split).1
    }
}

impl<B> From<Request<B>> for RoutedRequest<B> {
    fn from(request: Request<B>) -> Self {
        Self { request, split: 0 }
    }
}
