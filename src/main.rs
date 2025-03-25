pub struct QueuingPort<T> {
    buffer: [Option<T>; 16],        
    write_index: usize,      
    read_index: usize,       
}

impl<T> QueuingPort<T> {
    
    pub fn new() -> Self {
        Self {
            buffer: std::array::from_fn(|_| None),  
            write_index: 0,
            read_index: 0,
        }
    }

    pub fn enqueue(&mut self, item: T) -> Result<(), &str> {
        if self.is_full() {
            return Err("Buffer is full, cannot enqueue item.");
        }

        self.buffer[self.write_index] = Some(item);
        self.write_index = (self.write_index + 1) % self.buffer.len();
        Ok(())
    }

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

fn main() {
    let mut queue: QueuingPort<i32> = QueuingPort::new();

    queue.enqueue(100).unwrap();
    queue.enqueue(200).unwrap();
    queue.enqueue(300).unwrap();

    println!("Dequeued: {:?}", queue.dequeue().unwrap()); 
    println!("Dequeued: {:?}", queue.dequeue().unwrap()); 
    println!("Dequeued: {:?}", queue.dequeue().unwrap()); 
}

