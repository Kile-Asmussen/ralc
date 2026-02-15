# Reallocation-counting smart pointers

*RALC* is *R*e*AL*location-*C*ounting.

Ralc is a smart-pointer library with a strong/weak distinction based on using separately allocated metadata to track whether a pointer to an allocation is still valid.

An allocation gets designated a 'ledger' which stores the number of times allocations associated with this ledger has been deallocated, and also a read/write lock for acessing the allocation.

Each weak reference then stores the reallocation count for the ledger appointed to the allocation at time of creation, and can only access the allocated memory so long as the stored count matches the ledger. Should the strong reference be dropped, this reallocation count will be increased, invalidating all weak references.

Upon dropping the strong reference, the allocation and ledger assignment persists if there are active readers/writers. Only once the last reference is dropped does the memory deallocated and the ledger freed to be assigned elsewhere.

Compared to `Arc`, is has weak references that are `Copy`, only one strong reference per allocation, and a built-in read-write-lock access pattern. In exchange, it leaks small amounts memory, since the ledgers has to persist indefinitely.

Ralc also comes packaged with several different allocation arenas/scopes, similar to the difference between `Rc` and `Arc`:

- Ralcs can be allocated with `Sync` ledgers in a global arena, rendering any ralc of `Send`/`Sync` data globally sharable. These ledgers are leaked to `'static` lifetime.
- Ralcs can also be allocated with ledgers in a thread-local arena, which does not use any synchronized locking at all, for increased speed, but which are definitionally not sharable. Upon thread exit, all ledgers are properly deallocated.
- Lastly, direcly managed allocation pools are available, where all allocated ralcs are lifetime-gated to the pool.

All allocated ledgers live in properly managed allocators that allocate from free lists in the obvious fashion. Should a ledger ever reach `u64::MAX` allocations, it will be permanently leaked. This will probably not happen in practice, as `u64::MAX` nanoseconds is 500+ years.

# Dependencies

Ralc depends on Parking Lot for its management of global state, because the standard library does not offer the capacity to upgrade and downgrade permissions on read-write locks. This functionality is a hard requirement for the algorithm that ensures timely deallocation and ledger-reuse.

# Features

- `parking-lot`, enabled by default, uses Parking Lot to implement the global allocator. If this is disabled, global allocation isn't possible.
- `tokio` enables the use of Tokio's task-local data to implement an allocator analogous to the thread-local one.
- `bumpalo` use the much faster Bumpalo bump allocator library for allocation rather than standard library utilities.