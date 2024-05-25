use crate::alloc::Allocation;
use std::{
    any::Any,
    io::{stdin, stdout, Write},
};

static ALLOCATORS: [(&'static str, fn() -> Box<dyn Any>); 9] = [
    ("Stack allocation 1M", alloc::stack_allocate::<1>),
    //
    ("Heap zeroed 1M", alloc::heap_zero::<1>),
    ("Heap zeroed 10M", alloc::heap_zero::<10>),
    //
    ("Heap non-zero 1M", alloc::heap_non_zero::<1>),
    ("Heap non-zero 10M", alloc::heap_non_zero::<10>),
    //
    ("Heap uninitialized 1M", alloc::heap_uninit::<1>),
    ("Heap uninitialized 10M", alloc::heap_uninit::<10>),
    //
    ("Memory mapped 1M", alloc::mmap::<10>),
    ("Memory mapped 10M", alloc::mmap::<10>),
];

fn main() {
    let mut allocations: Vec<Allocation> = vec![];
    let mut error: Option<&'static str> = None;
    loop {
        print!("{}[2J", 27 as char);
        if !allocations.is_empty() {
            println!();
            println!("Current allocations:");
            for (i, allocation) in allocations.iter().enumerate() {
                println!("  {:>2}. {}", i + 1, allocation.0)
            }
            println!();
        }

        println!(
            "Create new allocation (1-{}) or remove current (-idx):",
            ALLOCATORS.len()
        );
        println!();
        for (idx, (name, _)) in ALLOCATORS.iter().enumerate() {
            println!("   {:2}. {}", idx + 1, name);
        }
        println!();

        if let Some(error) = error.take() {
            println!("  {}", error);
        }

        let mut string = String::new();
        print!("Choose allocator: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut string).unwrap();
        let trim = string.trim();
        if string.is_empty() || trim.eq_ignore_ascii_case("quit") {
            break;
        }
        let Ok(variant) = trim.parse::<i32>() else {
            error = Some("Invalid number");
            continue;
        };
        let idx = (variant.abs() - 1) as usize;
        match variant {
            x if x > 0 && idx < ALLOCATORS.len() => {
                let allocator = ALLOCATORS[idx];
                allocations.push(Allocation(allocator.0, allocator.1()));
            }
            x if x < 0 && idx < allocations.len() => {
                allocations.remove(idx);
            }
            _ => {
                error = Some("Invalid choice");
            }
        };
    }
}

mod alloc {
    use std::{
        any::Any,
        io::Write,
        mem::MaybeUninit,
        sync::{Arc, Barrier},
        thread::{self, JoinHandle},
    };

    use memmap::MmapOptions;
    use tempfile::tempfile;

    pub(super) struct Allocation(pub &'static str, #[allow(unused)] pub Box<dyn Any>);

    struct Stack<T>(Arc<Barrier>, Option<JoinHandle<T>>);

    impl<T> Drop for Stack<T> {
        fn drop(&mut self) {
            let Stack(barrier, thread) = self;
            if let Some(thread) = thread.take() {
                barrier.wait();
                thread.join().unwrap();
            }
        }
    }

    pub(super) fn stack_allocate<const SIZE_M: usize>() -> Box<dyn Any> {
        let barrier = Arc::new(Barrier::new(2));
        let join_handle = {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let v = vec![42u8; SIZE_M * 1024 * 1024];
                barrier.wait();
                // Needed to prevent variable to optimize out
                v.into_iter().sum::<u8>()
            })
        };
        Box::new(Stack(barrier, Some(join_handle)))
    }

    pub(super) fn heap_zero<const SIZE_M: usize>() -> Box<dyn Any> {
        Box::new(vec![0u8; SIZE_M * 1024 * 1024])
    }

    pub(super) fn heap_non_zero<const SIZE_M: usize>() -> Box<dyn Any> {
        let data = vec![42u8; SIZE_M * 1024 * 1024];
        Box::new(data)
    }

    pub(super) fn heap_uninit<const SIZE_M: usize>() -> Box<dyn Any> {
        let data = vec![MaybeUninit::<u8>::uninit(); SIZE_M * 1024 * 1024];
        Box::new(data)
    }

    pub(super) fn mmap<const SIZE_M: usize>() -> Box<dyn Any> {
        let mut file = tempfile().unwrap();
        let stride = vec![0u8; 1024];
        for _ in 0..(SIZE_M * 1024 * 1024) / stride.len() {
            file.write(&stride).unwrap();
        }
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        Box::new(mmap)
    }
}
