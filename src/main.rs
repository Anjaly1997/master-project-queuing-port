use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;
use once_cell::sync::Lazy;

// Struct definition for the QueuingPort
// #[repr(C)] is used to ensure the memory layout is consistent
#[repr(C)]  
pub struct QueuingPort<T> {
    buffer: [UnsafeCell<Option<T>>; 16],        
    write_index: AtomicUsize,      
    read_index: AtomicUsize,       
}

// Ensuring QueuingPort can be safely shared across threads
// UnsafeCell allows interior mutability of the buffer
unsafe impl<T> Sync for QueuingPort<T> where T: Send {}

impl<T> QueuingPort<T> {
    // Creates a new instance of QueuingPort with an empty buffer
    pub fn new() -> Self {
        Self {
            buffer: std::array::from_fn(|_| UnsafeCell::new(None)),  // Initializing buffer with None
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }

    // Adds an item to the buffer
    pub fn enqueue(&self, item: T) -> Result<(), &str> {
        // Load current write_index without requiring a memory barrier
        let write_index = self.write_index.load(Ordering::Relaxed);
        
        // Calculate the next index, wrapping around if needed
        let next_index = (write_index + 1) % self.buffer.len();

        // Check if buffer is full
        if next_index == self.read_index.load(Ordering::Acquire) {
            return Err("Buffer is full, cannot enqueue item.");
        }

        // Writing to the buffer (Unsafe because of interior mutability)
        unsafe {
            *self.buffer[write_index].get() = Some(item);
        }

        // Store the updated write index, making it visible to other threads
        self.write_index.store(next_index, Ordering::Release);
        Ok(())
    }

    // Removes an item from the buffer
    pub fn dequeue(&self) -> Result<T, &str> {
        // Load current read_index without requiring a memory barrier
        let read_index = self.read_index.load(Ordering::Relaxed);

        // Check if the buffer is empty
        if read_index == self.write_index.load(Ordering::Acquire) {
            return Err("Buffer is empty, cannot dequeue item.");
        }

        // Reading from the buffer (Unsafe because of interior mutability)
        let item = unsafe { 
            (*self.buffer[read_index].get()).take().ok_or("Failed to read item")?
        };

        // Move to the next index
        let next_index = (read_index + 1) % self.buffer.len();
        self.read_index.store(next_index, Ordering::Release);

        Ok(item)
    }
}

// Using Lazy to initialize the static shared queue only when accessed
static SHARED_QUEUE: Lazy<QueuingPort<i32>> = Lazy::new(|| QueuingPort::new());

impl QueuingPort<i32> {
    // Adds an item to the shared buffer
    pub fn enqueue_shared(item: i32) -> Result<(), &'static str> {
        SHARED_QUEUE.enqueue(item)
    }

    // Removes an item from the shared buffer
    pub fn dequeue_shared() -> Result<i32, &'static str> {
        SHARED_QUEUE.dequeue()
    }
}

fn main() {
    // Adding test items to the queue
    QueuingPort::enqueue_shared(100).unwrap();
    QueuingPort::enqueue_shared(200).unwrap();
    QueuingPort::enqueue_shared(300).unwrap();

    // Reading items from the queue
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
}
