#![cfg_attr(not(test), no_std)]  // Trying to enable no_std compatibility when not testing

use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use once_cell::sync::Lazy;  // Still using this, maybe it needs to be replaced for no_std?

// Struct definition for the QueuingPort
// #[repr(C)] should help maintain a predictable memory layout, right?
#[repr(C)]  
pub struct QueuingPort<T> {
    buffer: [UnsafeCell<Option<T>>; 16],        
    write_index: AtomicUsize,      
    read_index: AtomicUsize,       
}

// Trying to make QueuingPort thread-safe
unsafe impl<T> Sync for QueuingPort<T> where T: Send {}

impl<T> QueuingPort<T> {
    pub fn new() -> Self {
        Self {
            buffer: core::array::from_fn(|_| UnsafeCell::new(None)),  // Using core::array for no_std
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }

    pub fn enqueue(&self, item: T) -> Result<(), &str> {
        let write_index = self.write_index.load(Ordering::Relaxed);
        let next_index = (write_index + 1) % self.buffer.len();

        if next_index == self.read_index.load(Ordering::Acquire) {
            return Err("Buffer is full, cannot enqueue item.");
        }

        unsafe {
            *self.buffer[write_index].get() = Some(item);
        }

        self.write_index.store(next_index, Ordering::Release);
        Ok(())
    }

    pub fn dequeue(&self) -> Result<T, &str> {
        let read_index = self.read_index.load(Ordering::Relaxed);

        if read_index == self.write_index.load(Ordering::Acquire) {
            return Err("Buffer is empty, cannot dequeue item.");
        }

        let item = unsafe { 
            (*self.buffer[read_index].get()).take().ok_or("Failed to read item")?
        };

        let next_index = (read_index + 1) % self.buffer.len();
        self.read_index.store(next_index, Ordering::Release);

        Ok(item)
    }
}

//  using Lazy : incompatible with no_std. 
// TODO: Find a way to initialize static memory without Lazy, maybe use spin or something?
static SHARED_QUEUE: Lazy<QueuingPort<i32>> = Lazy::new(|| QueuingPort::new());

impl QueuingPort<i32> {
    pub fn enqueue_shared(item: i32) -> Result<(), &'static str> {
        SHARED_QUEUE.enqueue(item)
    }

    pub fn dequeue_shared() -> Result<i32, &'static str> {
        SHARED_QUEUE.dequeue()
    }
}

fn main() {
    // TODO: Test concurrency! Maybe use threads?
    QueuingPort::enqueue_shared(100).unwrap();
    QueuingPort::enqueue_shared(200).unwrap();
    QueuingPort::enqueue_shared(300).unwrap();

    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
}
