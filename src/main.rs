#[repr(C)]  
pub struct QueuingPort<T> {
    buffer: [Option<T>; 16],        
    write_index: usize,      
    read_index: usize,       
}

// Implementing the QueuingPort
impl<T> QueuingPort<T> {
    
   
    pub fn new() -> Self {
        Self {
            buffer: std::array::from_fn(|_| None),  
            write_index: 0,
            read_index: 0,
        }
    }

    //  (Enqueue operation)
    pub fn enqueue(&mut self, item: T) -> Result<(), &str> {
        if self.is_full() {
            return Err("Buffer is full, cannot enqueue item.");
        }

        self.buffer[self.write_index] = Some(item);
        self.write_index = (self.write_index + 1) % self.buffer.len();
        Ok(())
    }

    // (Dequeue operation)
    pub fn dequeue(&mut self) -> Result<T, &str> {
        if self.is_empty() {
            return Err("Buffer is empty, cannot dequeue item.");
        }

        let item = self.buffer[self.read_index].take().unwrap();
        self.read_index = (self.read_index + 1) % self.buffer.len();
        Ok(item)
    }


    fn is_full(&self) -> bool {
        (self.write_index + 1) % self.buffer.len() == self.read_index
    }

   
    fn is_empty(&self) -> bool {
        self.write_index == self.read_index && self.buffer[self.read_index].is_none()
    }
}


static mut SHARED_QUEUE: Option<QueuingPort<i32>> = None;


impl QueuingPort<i32> {

    // (Enqueue operation)
    pub fn enqueue_shared(item: i32) -> Result<(), &'static str> {
        unsafe {
            if let Some(ref mut queue) = SHARED_QUEUE {
                queue.enqueue(item)
            } else {
                Err("Shared memory is not initialized")
            }
        }
    }

    // (Dequeue operation)
    pub fn dequeue_shared() -> Result<i32, &'static str> {
        unsafe {
            if let Some(ref mut queue) = SHARED_QUEUE {
                queue.dequeue()
            } else {
                Err("Shared memory is not initialized")
            }
        }
    }
}

fn main() {
    // shared memory initilaisation
    unsafe { SHARED_QUEUE = Some(QueuingPort::new()); }

    // Test writing to shared memory
    QueuingPort::enqueue_shared(100).unwrap();
    QueuingPort::enqueue_shared(200).unwrap();
    QueuingPort::enqueue_shared(300).unwrap();

    // Test reading from shared memory
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared().unwrap()); 
}
