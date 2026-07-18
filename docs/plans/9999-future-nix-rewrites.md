A few I'd actually reach for, grouped by what they'd teach you:

**Performance rabbit holes**
- `wc` — looks trivial, becomes a SIMD/mmap playground. Fast line and word counting pulls you into `memchr`, `std::simd`, and cache-friendly chunking, and there's a whole genre of "can you beat GNU wc" writeups to benchmark against.
- `yes` — the joke that isn't. GNU `yes` pushes >10 GB/s by writing page-sized buffers, so the lesson is exactly why a naive `println!` loop is ~1000x slower. Tiny surface area, good warmup.

**Systems / OS-API tours**
- `tail -f` — the follow logic is the interesting bit: inotify/kqueue, handling truncation and log rotation, `--follow=name` vs `--follow=descriptor`. Sits right next to the watch work you've been doing.
- `xargs` — deceptively deep once you add `-P` parallelism. Argument batching, null-delimited `-0`, and an actual concurrency model to design (thread pool vs async).
- a mini `strace` — `ptrace`, `PTRACE_SYSCALL`, decoding syscall numbers and args out of registers. Linux-only and more work, but almost nothing else teaches the syscall boundary this viscerally.

**Algorithm-flavored**
- `sort` — the external merge sort version: spill to disk when input exceeds RAM, k-way merge, optional parallel or radix passes. The closest thing here to a database-internals exercise.
- `join` — essentially a merge join over two sorted files. Swap in a hash join and compare and you've got a small mirror of the planner tradeoffs you were poking at on the Postgres side.
- a tiny `grep` with a from-scratch regex engine — Thompson NFA construction and simulation (Russ Cox's regex series is the canonical guide). More automata theory than systems, if that's the itch.

If I had to pick one on best ratio of finishable-to-educational, it's `tail -f` if you want the OS-API flavor or `sort` if you want the algorithms. Want me to sketch a module layout or a rough scope for any of them?
