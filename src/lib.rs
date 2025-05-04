// Copyright 2025 Oleg Pavlenko

use std::{cell::Cell, error::Error, marker::PhantomData, time::{Duration, Instant}};
type FnThread = Box<dyn FnOnce() + 'static + Send>;

///
/// Builder for MiniPool
/// 
/// ```
/// fn main() {
///     let pool = MiniPool::builder()
///                             .count_threads(4)
///                             .build;
/// 
///     // ...
/// }
/// ```
/// 
#[derive(Default)]
pub struct MiniPoolBuilder {
        #[allow(dead_code)] // clippy never read fix
        senders: Option<Vec<std::sync::mpsc::Sender<FnThread>>>,
        #[allow(dead_code)] // clippy never read fix
        threads: Option<Vec<std::thread::JoinHandle<()>>>,
        balance: Option<Box<dyn Balancer>>,
        count_threads: Option<usize>,
        stack_size: Option<usize>
}

impl MiniPoolBuilder {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn count_threads(mut self, count: usize) -> Self {
        self.count_threads = Some(count);
        self
    }

    pub fn stack_size(mut self, bytes: usize) -> Self {
        self.stack_size = Some(bytes);
        self
    }

    pub fn balancer(mut self, balancer: impl Balancer + 'static) -> Self {
        self.balance = Some(Box::new(balancer));
        self
    }

    pub fn build(self) -> Result<MiniPool, Box<dyn Error>> {

        let count = self.count_threads.unwrap_or(std::thread::available_parallelism().expect("Not found count of threads").into());
        let mut threads = Vec::with_capacity(count.into());
        let mut senders = Vec::with_capacity(count.into());

        for _ in 0..count.into() {

            let (sx, rx) = std::sync::mpsc::channel::<FnThread>();
            senders.push(sx);

            let builder = if let Some(size) = self.stack_size {
                std::thread::Builder::new()
                    .stack_size(size)
            } else {
                std::thread::Builder::new()
            };

            threads.push(
                builder.spawn(move || {
                    while let Ok(func) = rx.recv() {
                        func();
                    }
                }).unwrap()
            );
        }

        Ok(MiniPool { phantom: PhantomData, senders, threads, balance: self.balance.unwrap_or(Box::new(DefaultBalancer{ index: Cell::new(0) })) })
    }
}

struct DefaultBalancer {
    index: Cell<usize>
}

impl Balancer for DefaultBalancer {
    fn index(&self, state: &MiniPool) -> usize {
        if self.index.clone().into_inner() == state.threads.len() { self.index.set(0); }
        let idx = self.index.clone().into_inner();
        self.index.set(idx + 1);
        idx
    }
}

pub trait Balancer {
    fn index(&self, state: &MiniPool) -> usize;
}

///
/// Create pool threads for execute parallel cpu bounds tasks
/// ```
/// let pool = MiniPool::new();
/// ```
pub struct MiniPool {
    // All senders for send function to a execute
    senders: Vec<std::sync::mpsc::Sender<FnThread>>,
    // All threads
    threads: Vec<std::thread::JoinHandle<()>>,
    // Balancer
    balance: Box<dyn Balancer>,
    // No Sync and Send
    phantom: PhantomData<*const ()>
} 

impl MiniPool {

    pub fn new() -> Self {
        MiniPoolBuilder::new().build().unwrap()
    }

    pub fn builder() -> MiniPoolBuilder {
        MiniPoolBuilder::new()
    }

    ///
    /// Сreates a new thread, but each new task is distributed between threads using a balancer
    /// ```
    /// fn main() {
    ///     let mut pool = Minipool::new();
    /// 
    ///     pool.spawn(|| {
    ///         for i in 0..1000 {
    ///             println!("First: {}", i);               
    ///         } 
    ///     });
    /// 
    ///     pool.spawn(|| {
    ///         for i in 0..1000 {
    ///             println!("Second: {}", i);  
    ///         }
    ///     })
    /// 
    ///     pool.join_all();
    /// }
    /// ```
    /// 
    pub fn spawn<F: FnOnce() + 'static + Send>(&self, func: F) {
        let index = self.balance.index(self);
        self.senders[index].send(Box::new(func)).unwrap();
    }

    /// 
    ///Creates a new thread and when the code execution duration is greater than the set value, resets the execution
    /// 
    /// ```
    /// fn main() {
    ///     let mut pool = Minipool::new();
    /// 
    ///     pool.spawn_with_timeout(|| {
    ///           std::thread::sleep(Duration::from_secs(60));
    ///           println!("end");
    ///     }, timeout: Duration::from_secs(3));
    /// 
    ///     pool.join_all();
    /// }
    /// ```
    /// 
    pub fn spawn_with_timeout<F: FnOnce() + 'static + Send>(&mut self, func: F, timeout: Duration) {
        self.spawn(move || {
            let time = Instant::now();
            let handle = std::thread::spawn(func);
            loop {
                if time.elapsed() > timeout { break; }
                if handle.is_finished() { break; }
            }
        })
    }

    ///
    /// Blocks the main thread until all running threads have completed.
    /// 
    /// ```
    /// fn main() {
    ///     let mut pool = Minipool::new();
    /// 
    ///     pool.spawn(|| {
    ///           some_function_1();
    ///     });
    /// 
    ///     pool.spawn(|| {
    ///         some_function_2();
    ///     });
    /// 
    ///     pool.join_all();
    /// }
    /// ```
    /// 
    pub fn join_all(&mut self) {

        self.senders.clear();

        loop {
            let mut is_finish = true;
            for i in 0..self.threads.len() {
                if !self.threads[i].is_finished() {
                    is_finish = false;
                }
            }

            if is_finish { break; }
        }
    }
}


#[test]
fn test() {

    use std::sync::*;

    let pool = MiniPoolBuilder::new()
                                    .count_threads(4)
                                    .stack_size(1024)
                                    .build();

    assert_eq!(true, pool.is_ok());

    let m = Arc::new(Mutex::new(0));
    let mut pool = MiniPool::new();

    let n = m.clone();
    pool.spawn(move || { *n.lock().unwrap() = 1  });
    pool.join_all();

    assert_eq!(1, *(m.lock().unwrap()))

}