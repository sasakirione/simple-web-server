use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use log::error;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A pool of worker threads for executing tasks concurrently
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool with the specified number of threads
    ///
    /// # Arguments
    ///
    /// * `size` - The number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "Thread pool size must be greater than 0");

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Execute a job in the thread pool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // Handle the case where sender is None
        match self.sender.as_ref() {
            Some(sender) => {
                // Handle errors from send()
                if let Err(e) = sender.send(job) {
                    error!("Failed to send job to thread pool: {}", e);
                }
            },
            None => {
                error!("Thread pool sender is None, cannot execute job");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to signal workers to stop
        drop(self.sender.take());

        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // Handle errors from thread.join()
                if let Err(e) = thread.join() {
                    error!("Failed to join worker thread {}: {:?}", worker.id, e);
                }
            }
        }
    }
}

/// A worker thread that executes jobs from the thread pool
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Lock the receiver and try to get a job
            let message = match receiver.lock() {
                Ok(lock) => lock.recv(),
                Err(e) => {
                    // Mutex is poisoned, log error and exit thread
                    error!("Failed to lock receiver in worker thread: {:?}", e);
                    break;
                }
            };

            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    // Channel is closed, time to exit
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
