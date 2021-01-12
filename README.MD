Intro:

This is a recreation of the project boostCPPReader project, with the extended regex features and more involved paralellization than that program. 

It's important to note that this project is much faster than boostCPPreader *Even considering it sorts the results* showcasing the benefits rust has over C++.

In the C++ version it explicitly has a high level blocking function, which I am not aware of the inner workings of, based around the boost library. This has a cost associated with it in C++ and does in fact have overhead and runtime.
------>
    ------>
        ------>
This means it can only be as fast as the slowest file & additionally is impacted by the overhead in the main thread to allocate the pool. 

in the Rust Version, there is no high level resource allocation to generate the pool. It's 0 cost abstraction. There is some fancy work that goes into cross-beam to make this happen. It is in fact, pretty cool.

Rust also shares, in this instance, the synchronous limitation, but only in my own implementation. Rust actually supports fully asynchronous programming with async-std and Tokio, this would utterly destroy both implementations of this project by taking advantage of asynchronous processing and the ability to process while waiting for the ability to write to disk, etc, instead of waiting.

