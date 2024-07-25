# About
Brainfuck interpreter written in Rust

# Usage
` cargo run --release <file_name.bf> `

### Example
` cargo run --release examples/helloworld.bf `


# References
1. Wikipedia - https://en.wikipedia.org/wiki/Brainfuck#Hello_World!
2. Gist from GitHub User roachhd - https://gist.github.com/roachhd/dce54bec8ba55fb17d3a


# Ideas if ever look at this again
1. Add more optimizations for looped operations
2. Look into assembly generations, libc calls, and implement JIT compiler
3. Include more interesting and complicated examples