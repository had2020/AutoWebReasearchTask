Licensed under either of Apache-2.0 or MIT at your option.

Contribution from everyone is well welcomed in the form of suggestions, bug reports, pull requests, and feedback.

This project prioritizes mechanical sympathy and microarchitectural correctness.

    Performance: PRs should maintain L1/L2 cache residency and use Data-Oriented Design (64-byte alignment).

    Efficiency: Minimize branches and break data dependencies to maximize ILP.

    Allocations: Zero-copy is the standard; avoid the heap in hot paths.

Legal

All contributions are dual-licensed under MIT/Apache-2.0. This project is a non-commercial, independent effort.
