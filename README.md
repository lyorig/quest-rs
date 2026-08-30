# quest-rs
This is a Rust rewrite of my [Quest](https://github.com/lyorig/Quest) project.
Since Rust has many quality-of-life features compared to (and thanks to the mistakes of) C++, I thought it worthwhile to rewrite the existing code, and continue development in this language.

Since then, it's become sort of a playground for halcyon-rs, where I can find out what API design decisions work in practice.

## Game resources
The game expects files to be in a directory specified by [SDL_GetPrefPath](https://wiki.libsdl.org/SDL3/SDL_GetPrefPath), where `org` is "cz.lyorig", and `app` is "quest".
