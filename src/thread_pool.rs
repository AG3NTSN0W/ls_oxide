use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};


type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJoB(Job),
    Terminate
}

pub struct ThreadPool {
    pub workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // println!("Creating {} threads", size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // creat threads
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJoB(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // println!("Terminate all workers");
        
        for _ in &self.workers  {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            // println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


#[allow(dead_code)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJoB(job) => {
                    // println!("Worker {} got a job; executing.", id);
                    job()
                },
                Message::Terminate => {
                    // println!("Terminating thread: {}", id);
                    break;
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}