# launchred

Make your stderr red.

[`stderred`](https://github.com/sickill/stderred) is an interesting program, but it relies on LD_PRELOAD to function. `launchered` just launches a program as a child process and redirects its stderr, a simpler & more portable solution (for example, `launchred` works fine with static binaries).
