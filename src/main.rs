use crate::alloc::Allocation;
use std::{
    any::Any,
    io::{stdin, stdout, Write},
};
type Allocator = (&'static str, fn() -> Box<dyn Any>);
static ALLOCATORS: [Allocator; 13] = [
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
    ("File mmap 1M", alloc::mmap_file::<1>),
    ("File mmap 10M", alloc::mmap_file::<10>),
    //
    ("Anonymous mmap 1M", alloc::mmap_anon::<1>),
    ("Anonymous mmap 10M", alloc::mmap_anon::<10>),
    //
    ("Anonymous mmap dirty 1M", alloc::mmap_anon_init::<1>),
    ("Anonymous mmap dirty 10M", alloc::mmap_anon_init::<10>),
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
    use memmap::MmapOptions;
    use std::{
        any::Any,
        io::Write,
        mem::MaybeUninit,
        sync::{Arc, Barrier},
        thread::{self, JoinHandle},
        usize,
    };
    use tempfile::tempfile;
    const MEGABYTES: usize = 1024 * 1204;

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
                let v = vec![42u8; SIZE_M * MEGABYTES];
                barrier.wait();
                // Needed to prevent variable to optimize out
                v.into_iter().sum::<u8>()
            })
        };
        Box::new(Stack(barrier, Some(join_handle)))
    }

    /// Allocates zeroed memory on the heap
    pub(super) fn heap_zero<const SIZE_M: usize>() -> Box<dyn Any> {
        Box::new(vec![0u8; SIZE_M * MEGABYTES])
    }

    /// Allocates memory on the heap initialized with non-zero value
    ///
    /// It matters, because on some OSes zeroed memory might not be resident
    pub(super) fn heap_non_zero<const SIZE_M: usize>() -> Box<dyn Any> {
        let data = vec![42u8; SIZE_M * MEGABYTES];
        Box::new(data)
    }

    /// Allocated uninitialized memory on the heap
    pub(super) fn heap_uninit<const SIZE_M: usize>() -> Box<dyn Any> {
        let data = vec![MaybeUninit::<u8>::uninit(); SIZE_M * MEGABYTES];
        Box::new(data)
    }

    /// Memory map a file of the given size
    ///
    /// File is removed immediately
    pub(super) fn mmap_file<const SIZE_M: usize>() -> Box<dyn Any> {
        let mut file = tempfile().unwrap();
        let stride = vec![0u8; 1024];
        for _ in 0..(SIZE_M * 1024 * 1024) / stride.len() {
            file.write_all(&stride).unwrap();
        }
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        Box::new(mmap)
    }

    /// Memory map anonymous memory of the given size
    pub(super) fn mmap_anon<const SIZE_M: usize>() -> Box<dyn Any> {
        let mmap = MmapOptions::new()
            .len(SIZE_M * MEGABYTES)
            .map_anon()
            .unwrap();
        Box::new(mmap)
    }

    /// Memory map anonymous uninitialized allocation
    pub(super) fn mmap_anon_init<const SIZE_M: usize>() -> Box<dyn Any> {
        let size = SIZE_M * MEGABYTES;
        let mut mmap = MmapOptions::new().len(size).map_anon().unwrap();
        mmap[..size].fill(42);
        Box::new(mmap)
    }
}
