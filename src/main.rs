
pub struct QueuingPort<T> {
    buffer: [Option<T>; 16],        
    write_index: usize,      
    read_index: usize,       
}

impl<T> QueuingPort<T> {
    
    pub fn new() -> Self {
        Self {
            buffer: [None; 16],  
            write_index: 0,
            read_index: 0,
        }
    }
}
