#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use crate::thread_pool::ThreadPool;

    #[test]
    fn test_thread_pool_creation() {
        let _pool = ThreadPool::new(4);
        // Just testing that it doesn't panic
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "Thread pool size must be greater than 0")]
    fn test_thread_pool_zero_size() {
        let _pool = ThreadPool::new(0);
    }

    #[test]
    fn test_thread_pool_execute_single_task() {
        let pool = ThreadPool::new(1);

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        pool.execute(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });

        // Give some time for the task to complete
        thread::sleep(Duration::from_millis(100));

        let result = *counter.lock().unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_thread_pool_execute_multiple_tasks() {
        let pool = ThreadPool::new(4);

        let counter = Arc::new(Mutex::new(0));
        let num_tasks = 10;

        for _ in 0..num_tasks {
            let counter_clone = Arc::clone(&counter);
            pool.execute(move || {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            });
        }

        // Give some time for all tasks to complete
        thread::sleep(Duration::from_millis(200));

        let result = *counter.lock().unwrap();
        assert_eq!(result, num_tasks);
    }

    #[test]
    fn test_thread_pool_drop() {
        let counter = Arc::new(Mutex::new(0));

        {
            let pool = ThreadPool::new(2);

            for _ in 0..5 {
                let counter_clone = Arc::clone(&counter);
                pool.execute(move || {
                    // Simulate some work
                    thread::sleep(Duration::from_millis(50));
                    let mut num = counter_clone.lock().unwrap();
                    *num += 1;
                });
            }

            // Pool will be dropped at the end of this scope
        }

        // Give some time for all tasks to complete before the pool is dropped
        thread::sleep(Duration::from_millis(300));

        // All tasks should have completed
        let result = *counter.lock().unwrap();
        assert_eq!(result, 5);
    }
}
