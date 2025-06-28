#![cfg_attr(not(test), no_std)]

use core::sync::atomic::{AtomicUsize, Ordering};
use core::mem::size_of;

#[cfg(test)]
extern crate std;

#[cfg(test)]
use std::thread;

use shared_memory::{Shmem, ShmemConf};

const MSG_COUNT: usize = 16;
const MSG_SIZE: usize = size_of::<i32>();

#[repr(C)]
pub struct QueuingPort {
    buffer: [u8; MSG_COUNT * MSG_SIZE],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
}

unsafe impl Sync for QueuingPort {}

impl QueuingPort {
    pub fn new() -> Self {
        Self {
            buffer: [0; MSG_COUNT * MSG_SIZE],
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }

    pub fn enqueue(&self, item: i32) -> Result<(), &'static str> {
        let write = self.write_index.load(Ordering::Relaxed);
        let next = (write + 1) % MSG_COUNT;

        if next == self.read_index.load(Ordering::Acquire) {
            return Err("Queue full");
        }

        let offset = write * MSG_SIZE;
        let ptr = &self.buffer[offset] as *const u8 as *mut i32;
        unsafe { ptr.write(item); }

        self.write_index.store(next, Ordering::Release);
        Ok(())
    }

    pub fn dequeue(&self) -> Result<i32, &'static str> {
        let read = self.read_index.load(Ordering::Relaxed);

        if read == self.write_index.load(Ordering::Acquire) {
            return Err("Queue empty");
        }

        let offset = read * MSG_SIZE;
        let ptr = &self.buffer[offset] as *const u8 as *const i32;
        let value = unsafe { ptr.read() };

        self.read_index.store((read + 1) % MSG_COUNT, Ordering::Release);
        Ok(value)
    }
}

// === Shared Memory Setup ===

static mut SHARED_QUEUE_PTR: *mut QueuingPort = core::ptr::null_mut();
static mut SHMEM_HANDLE: Option<Shmem> = None;

fn get_shared_queue(os_id: &str) -> &'static mut QueuingPort {
    unsafe {
        if !SHARED_QUEUE_PTR.is_null() {
            return &mut *SHARED_QUEUE_PTR;
        }

        let size = size_of::<QueuingPort>();
        let shmem = ShmemConf::new()
            .size(size)
            .os_id(os_id)
            .create()
            .expect("Failed to create shared memory");

        let ptr = shmem.as_ptr() as *mut QueuingPort;
        ptr.write(QueuingPort::new());

        SHMEM_HANDLE = Some(shmem);
        SHARED_QUEUE_PTR = ptr;

        &mut *ptr
    }
}

// === Public API ===

impl QueuingPort {
    pub fn enqueue_shared(item: i32, os_id: &str) -> Result<(), &'static str> {
        get_shared_queue(os_id).enqueue(item)
    }

    pub fn dequeue_shared(os_id: &str) -> Result<i32, &'static str> {
        get_shared_queue(os_id).dequeue()
    }
}

// === Main Function ===

#[cfg(feature = "std")]
fn main() {
    let os_id = "main_queue";

    QueuingPort::enqueue_shared(100, os_id).unwrap();
    QueuingPort::enqueue_shared(200, os_id).unwrap();
    QueuingPort::enqueue_shared(300, os_id).unwrap();

    println!("Dequeued: {:?}", QueuingPort::dequeue_shared(os_id).unwrap());
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared(os_id).unwrap());
    println!("Dequeued: {:?}", QueuingPort::dequeue_shared(os_id).unwrap());
}

#[cfg(not(feature = "std"))]
fn main() {
    // no-op for embedded/no_std
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_enqueue_dequeue_shared() {
        let os_id = "test_queue_1";

        QueuingPort::enqueue_shared(10, os_id).unwrap();
        QueuingPort::enqueue_shared(20, os_id).unwrap();

        let x = QueuingPort::dequeue_shared(os_id).unwrap();
        let y = QueuingPort::dequeue_shared(os_id).unwrap();

        println!("Dequeued values: {}, {}", x, y);
        assert_eq!(x, 10);
        assert_eq!(y, 20);
    }

    #[test]
    fn test_single_writer_reader() {
    let os_id = "test_queue_fixed";

    //writer thread
    let writer = thread::spawn({
        let id = os_id.to_string();
        move || {
            for i in 0..10 {
                let _ = QueuingPort::enqueue_shared(i, &id);
            }
        }
    });

    writer.join().unwrap();

    // reader thread
    let reader = thread::spawn({
        let id = os_id.to_string();
        move || {
            let mut results = vec![];
            for _ in 0..10 {
                if let Ok(val) = QueuingPort::dequeue_shared(&id) {
                    results.push(val);
                }
            }

            results.sort();
            assert_eq!(results, (0..10).collect::<Vec<_>>());
        }
    });

    reader.join().unwrap();
}
}
