use std::{
    any::Any,
    io::{stdin, stdout, Write},
    mem::MaybeUninit,
    sync::{Arc, Barrier},
    thread::{self, JoinHandle},
};

struct Allocation(&'static str, #[allow(unused)] Box<dyn Any>);

impl<T: 'static> Into<Allocation> for (&'static str, T) {
    fn into(self) -> Allocation {
        let (name, value) = self;
        Allocation(name, Box::new(value))
    }
}

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

fn stack_allocate<const SIZE_M: usize>() -> Box<dyn Any> {
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

fn heap_zeroed<const SIZE_M: usize>() -> Box<dyn Any> {
    Box::new(vec![0u8; SIZE_M * 1024 * 1024])
}

fn heap_non_zero<const SIZE_M: usize>() -> Box<dyn Any> {
    let data = vec![42u8; SIZE_M * 1024 * 1024];
    Box::new(data)
}

fn heap_uninitialized<const SIZE_M: usize>() -> Box<dyn Any> {
    let data = vec![MaybeUninit::<u8>::uninit(); SIZE_M * 1024 * 1024];
    Box::new(data)
}

static ALLOCATIONS_TYPES: [(&'static str, fn() -> Box<dyn Any>); 10] = [
    ("Stack allocation 1M", stack_allocate::<1>),
    //
    ("Heap zeroed 1M", heap_zeroed::<1>),
    ("Heap zeroed 10M", heap_zeroed::<10>),
    ("Heap zeroed 100M", heap_zeroed::<100>),
    //
    ("Heap non-zero 1M", heap_non_zero::<1>),
    ("Heap non-zero 10M", heap_non_zero::<10>),
    ("Heap non-zero 100M", heap_non_zero::<100>),
    //
    ("Heap uninitialized 1M", heap_uninitialized::<1>),
    ("Heap uninitialized 10M", heap_uninitialized::<10>),
    ("Heap uninitialized 100M", heap_uninitialized::<100>),
];

fn main() {
    let mut curent_allocations: Vec<Allocation> = vec![];
    let mut error: Option<&'static str> = None;
    loop {
        print!("{}[2J", 27 as char);
        if !curent_allocations.is_empty() {
            println!();
            println!("Current allocations:");
            for (i, allocation) in curent_allocations.iter().enumerate() {
                println!("  {:>2}. {}", i + 1, allocation.0)
            }
            println!();
        }

        println!(
            "Create new allocation (1-{}) or remove current (-idx):",
            ALLOCATIONS_TYPES.len()
        );
        println!();
        for (idx, (name, _)) in ALLOCATIONS_TYPES.iter().enumerate() {
            println!("   {:2}. {}", idx + 1, name);
        }
        println!();

        if let Some(error) = error.take() {
            println!("  {}", error);
        }

        let mut string = String::new();
        print!("Choose allocation: ");
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
            x if x > 0 && idx < ALLOCATIONS_TYPES.len() => {
                let allocation_type = ALLOCATIONS_TYPES[idx];
                curent_allocations.push(Allocation(allocation_type.0, allocation_type.1()));
            }
            x if x < 0 && idx < curent_allocations.len() => {
                curent_allocations.remove(idx);
            }
            _ => {
                error = Some("Invalid choice");
            }
        };
    }
}
