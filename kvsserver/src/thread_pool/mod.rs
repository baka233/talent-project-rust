
/// The trait provides the basic operation of the threadpool,
pub struct ThreadPool {

    /// To build a new ThreadPool.
    fn new() -> Self;

    /// spwan use channel to communicated with each other.
    fn spawn<F>(&self, job : F) 
    where
        F : FnOnce() + Send + 'static;
}

