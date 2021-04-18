# Spelltower Solver

This short Rust script is just a proof of concept for finding all words inside of a spelltower "search" game. I used it
to teach myself OCR, a bit more rust and threading.

I have also included a slightly modified list of words based on https://github.com/dwyl/english-words. I have removed
all 1 and 2 letter words since they don't count in Spelltower. Some of these words aren't identified as being valid
words by Spelltower's internal dictionary.

### Requirements (besides Cargo)

* Tesseract/Leptonica - I only got this working trying to build the code, having cargo complain about a particular
  package and installing the vcpkg package it tells me. See:  https://github.com/microsoft/vcpkg
  
### Use

You will require a screenshot of a Spelltower board. On the iPhone, you can just hit volume up and power at the same
time to take a screenshot.

`cargo run -- <path to dictionary> <path to spelltower screenshot> <optional algorithm selector>
{}"`

or

`./spelltower-solver <path to dictionary> <path to spelltower screenshot> <optional algorithm selector>
{}"`

Algorithm options:

* 0 - Multithreaded map/reduce - Default
* 1 - Multithreaded with Mutex based HashSet
* 2 - Single threaded with HashSet

Multithreaded map/reduce is the fastest algorithm.

### Next Steps

This is purely a POC but I might continue to play around with this and include a potential wasm target since both Rayon
and Tesseract do have some wasm forks. Unclear how well they work yet.

I also would like to play with caching or techniques like search pruning to see if there's ways to make this script even
more effective.

I could also see injecting additional logic to actually taking into account bonus letters and the rules of the search
game mode as well as supporting additional game modes.