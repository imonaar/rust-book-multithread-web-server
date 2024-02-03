use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        /*
            mpsc - Multi-producer, single-consumer

            - only one receiver is supported, but we to send data to all the threads we spawned
            - but we cant clone the receiver

            so how do we handle a situation where we need 'multiple receivers'
            -- intoduce a mutex
            mutex - A mutual exclusion primitive useful for protecting shared data
            This mutex will block threads waiting for the lock to become available.

            why Arc? -> to make is thread safe to access from multiple threads
            arc - Atomically Reference Counted
            Invoking clone on Arc produces a new Arc instance, which points to the same allocation on the heap as the source Arc
        */
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            //For each new worker, we clone the Arc to bump the reference count so the workers can share ownership of the receiver.
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /*
        execute takes a closure that implements FnOnce() send and static
        FnOnce -> a type that implements FnOnce can only be called once, we want our handleConnection
        to be called only once in this new thread ?
        f consumes its captured varibales so it cannot be run more than once!

        -- Use FnOnce as a bound when you want to accept a parameter of function-like type and
        -- only need to call it once. If you need to call the parameter repeatedly,
        -- use FnMut as a bound; if you also need it to not mutate state, use Fn.

        The Send marker trait indicates that ownership of the type implementing Send can be transferred
        between threads.

    */
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Box - allocation on the heap & not on the stack.
        let job = Box::new(f);

        //send the closure i.e function to be executed by the thread bool
        self.sender.as_ref().unwrap().send(job).unwrap()
        /*
            There is a little bit of indirection in this code?
            Where does send send its job? Looking at it from the new function is confusing;
            Threadpool::new creates a vector with size workers, this workers are standby and readt to receive a job
            -- the receiver is common in all the workers & protected by a mutex ensuring only one worker
                can receive a job at a given time!

            --so, we are sending the job to the worker pulll.
        */
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        /*
           drop the sender first  before dropping the workers
           Dropping sender closes the channel, which indicates no more messages will be sent
           When that happens, all the calls to recv that the workers do in the infinite loop will return an error
        */
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            }
        }
    }
}

/*
    External code (like our server in src/main.rs) doesn’t need to know the implementation details regarding
    using a Worker struct within ThreadPool, so we make the Worker struct and its new function private.
*/
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
    // thread: thread::JoinHandle<Arc<Mutex<mpsc::Receiver<Job>>>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // If the operating system can’t create a thread because there aren’t enough system resources, thread::spawn will panic.
        let thread = thread::spawn(move || loop {
            // /*
            //     If we get the lock on the mutex, we call recv to receive a Job from the channel.
            //     The call to recv blocks, so if there is no job yet, the current thread will wait until a job becomes available.
            //     The Mutex<T> ensures that only one Worker thread at a time is trying to request a job
            // */
            // let job = receiver.lock().unwrap().recv().unwrap();
            // println!("Worker {id} got a job; executing.");
            // job() // This is our handle_connection(stream) in this case

            // //we loop to keep the thread alive and wait for other jobs

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
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
