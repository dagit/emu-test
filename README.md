# emu-test
timing different ways to implement a cycle accurate emulator

This is based on doing some explorations inspired by this post: https://byuu.net/design/cooperative-threading

* The `null` variant doesn't attempt to do anything other than cycle counting.
* The `enum` variant uses a very simple encoding of the state machine as integer values. It's a pain to program in and only exists to have something to compare against.
* The `genawaiter` variant uses the genawaiter library to achieve a coroutine sort of thing. That code is doing the right thing in my test case, but I'm a little skeptical of my `wait` function, see `local_join`.
* The `tokio` variant uses tokio in what I think is the most straight forward way?
* The `async-std` variant is just like the tokio variant but using `async-std` instead of tokio.

Not shown in this repo is a simple example in C that I created using byuu's `libco` library. That library only provides the task switch so I also needed to make a simple scheduler. That version can do 5 million instructions in 1second on my machine. For comparison, `genawaiter` is 4s for the same workload, and `tokio` is about 12 seconds.
