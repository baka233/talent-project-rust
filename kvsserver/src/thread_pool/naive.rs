

/// This ThreadPool just to create a new thread 
/// when an request has happened;
pub struct NaiveThreadPool;


impl ThreadPool for NaiveThreadPool {
    fn new() -> Self {
        NaiveThreadPool;         
    }

    /// just creat new spawn
    fn spawn<F>(&self, func : F)
    where 
        F : FnOnce() + Send + 'static
    {
        thread::spawn(func);
    }
}

