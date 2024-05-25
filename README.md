This utility is designed to help understand how memory is handled by different operating systems.
It provides various types of memory allocations and allows to create and remove these allocations
interactively. The utility supports different types of memory allocations such as stack allocation,
heap allocation (zeroed, non-zeroed, and uninitialized), and memory-mapped files.

The purpose of this utility is to assist in interpreting numbers reported by system utilities like
`pmap`, `vmmap`, and others. By creating different types of allocations and observing how they are
reported by these system utilities, users can gain insights into how memory is managed by the operating
system.

# Usage

When running the utility, users are presented with a menu to create new allocations or remove existing
ones. The menu lists different types of allocations that can be created. Users can choose an allocator
by entering its corresponding number or remove an existing allocation by entering its negative index.
The current allocations are displayed along with their indices, allowing users to manage the allocations
interactively.

# Example

```
Create new allocation (1-9) or remove current (-idx):
   1. Stack allocation 1M
   2. Heap zeroed 1M
   3. Heap zeroed 10M
   4. Heap non-zero 1M
   5. Heap non-zero 10M
   6. Heap uninitialized 1M
   7. Heap uninitialized 10M
   8. Memory mapped 1M
   9. Memory mapped 10M
```

Users can enter a number to create a new allocation or a negative number to remove an existing allocation.
For example, entering `1` will create a stack allocation of 1MB, and entering `-1` will remove the first
allocation in the current list.

# Allocators

- Stack allocation: Allocates memory on the stack using a separate thread to prevent optimization.
- Heap zeroed: Allocates zeroed memory on the heap.
- Heap non-zero: Allocates memory on the heap initialized with non-zero value to prevent hardware zeroing.
- Heap uninitialized: Allocates uninitialized memory on the heap.
- Memory mapped: Allocates memory using memory-mapped files.
