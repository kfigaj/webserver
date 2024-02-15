use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use thiserror::Error;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;


impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker {id, thread:Some(thread)}
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "ThreadPool should have size bigger than 0");

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            // create some threads and store them in the vector
            workers.push(Worker::new(i, Arc::clone(&receiver)));

        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadPool;

    #[test]
    #[should_panic(expected = "ThreadPool should have size bigger than 0")]
    fn thread_panic_for_zero() {
        ThreadPool::new(0);
    }

    #[test]
    fn thread_successful_one_thread() {
        ThreadPool::new(1);
    }

    #[test]
    fn thread_successful_multiple_thread() {
        ThreadPool::new(4);
    }

    #[test]
    fn thread_execute_correctly() {
        let pool = ThreadPool::new(4);
        pool.execute(|| {
            for number in 0..4 {
                println!("{number}!");
            }
        })
    }


}