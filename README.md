[![Rust](https://github.com/Byron/dua-cli/workflows/Rust/badge.svg)](https://github.com/byron/dua-cli/actions)
[![Crates.io](https://img.shields.io/crates/v/dua-cli.svg)](https://crates.io/crates/dua-cli)
[![Packaging status](https://repology.org/badge/tiny-repos/dua-cli.svg)](https://repology.org/project/dua-cli/badges)

**dua** (-> _Disk Usage Analyzer_) is a tool to conveniently learn about the usage of disk space of a given directory. It's parallel by default and will max out your SSD, providing relevant information as fast as possible. Optionally delete superfluous data, and do so more quickly than `rm`.

[![asciicast](https://asciinema.org/a/316444.svg)](https://asciinema.org/a/316444)

### Installation

### Binary Release 

#### MacOS

```sh
curl -LSfs https://raw.githubusercontent.com/Byron/dua-cli/master/ci/install.sh | \
    sh -s -- --git Byron/dua-cli --crate dua --tag v2.17.4
```

#### MacOS via [MacPorts](https://www.macports.org):
```sh
sudo port selfupdate
sudo port install dua-cli
```

#### MacOS via [Homebrew](https://brew.sh)
```sh
brew update
brew install dua-cli
```

#### Linux

Linux requires the target to be specified explicitly to obtain the MUSL build.

```sh
curl -LSfs https://raw.githubusercontent.com/Byron/dua-cli/master/ci/install.sh | \
    sh -s -- --git Byron/dua-cli --target x86_64-unknown-linux-musl --crate dua --tag v2.17.4
```

#### Windows via [Scoop](https://scoop.sh/)
```sh
scoop install dua
```

#### Pre-built Binaries

See the [releases section][releases] for manual installation of a binary, pre-built for many platforms.

[releases]: https://github.com/Byron/dua-cli/releases

#### Cargo
Via `cargo`, which can be obtained using [rustup][rustup]

For _Unix_…
```
cargo install dua-cli

# And if you don't need a terminal user interface (most compatible)
cargo install dua-cli --no-default-features

# Compiles on most platforms, with terminal user interface
cargo install dua-cli --no-default-features --features tui-crossplatform
```

For _Windows_, nightly features are currently required.
```
cargo +nightly install dua-cli
```

#### VoidLinux
Via `xbps` on your VoidLinux system.

```
xbps-install dua-cli
```

#### Fedora
Via `dnf` on your Fedora system.

```
sudo dnf install dua-cli
```

#### Arch Linux
Via `pacman` on your ArchLinux system.

```
sudo pacman -S dua-cli
```

#### NetBSD
Via `pkgin` on your NetBSD system.

```
pkgin install dua-cli
```

Or, building from source

```
cd /usr/pkgsrc/sysutils/dua-cli
make install
```

#### Windows

You will find pre-built binaries for Windows in the [releases section][releases].
Alternatively, install via cargo as in

```
cargo +nightly install dua-cli
```

### Usage

```bash
# count the space used in the current working directory
dua
# count the space used in all directories that are not hidden
dua *
# learn about additional functionality
dua aggregate --help
```

### Interactive Mode

Launch into interactive mode with the `i` or `interactive` subcommand. Get help on keyboard
shortcuts with `?`.
Use this mode to explore, and/or to delete files and directories to release disk space.

Please note that great care has been taken to prevent accidential deletions due to a multi-stage
process, which makes this mode viable for exploration.

```bash
dua i
dua interactive
```

### Development

Please note that all the following assumes a unix system. On Windows, the linux subsystem should do the job.

#### Run tests

```bash
make tests
```

#### Learn about other targets

```
make
```

#### But why is…

#### …there two available backends? `crossterm` works everywhere!

Why add complexity to support `termion` if `crossterm` works everywhere? The answer is compile time and binary size, which both are larger
when using `crossterm`. Thus on Unix we still build with `termion`, but there is no reason to stop supporting it once `crossterm` has no
disadvantages.

The `crosstermion` crate makes handling this a bit less cumbersome.

### Acknowledgements

Thanks to [jwalk][jwalk], all there was left to do is to write a command-line interface. As `jwalk` matures, **dua** should benefit instantly.

### Limitations

* Does not show symbolic links at all if no path is provided when invoking `dua`
  * in an effort to skip symbolic links, for now there are pruned and are not used as a root. Symbolic links will be shown if they
    are not a traversal root, but will not be followed.
* Interactive mode only looks good in dark terminals (see [this issue](https://github.com/Byron/dua-cli/issues/13))
* _easy fix_: file names in main window are not truncated if too large. They are cut off on the right.
* There are plenty of examples in `tests/fixtures` which don't render correctly in interactive mode.
  This can be due to graphemes not interpreted correctly. With Chinese characters for instance,
  column sizes are not correctly computed, leading to certain columns not being shown.
  In other cases, the terminal gets things wrong - I use alacritty, and with certain characaters it
  performs worse than, say iTerm3.
  See https://github.com/minimaxir/big-list-of-naughty-strings/blob/master/blns.txt for the source.
* In interactive mode, you will need about 60MB of memory for 1 million entries in the graph.
* In interactive mode, the maximum amount of files is limited to 2^32 - 1 (`u32::max_value() - 1`) entries.
  * One node is used as to 'virtual' root
  * The actual amount of nodes stored might be lower, as there might be more edges than nodes, which are also limited by a `u32` (I guess)
  * The limitation is imposed by the underlying [`petgraph`][petgraph] crate, which declares it as `unsafe` to use u64 for instance.
  * It's possibly *UB* when that limit is reached, however, it was never observed either.

### Similar Programs 

* **CLI:**
  * `du`
  * [`dust`](https://github.com/bootandy/dust)
  * [`dutree`](https://github.com/nachoparker/dutree)
  * [`pdu`](https://github.com/KSXGitHub/parallel-disk-usage)
* **TUI:**
  * [`ncdu`](https://dev.yorhel.nl/ncdu)
  * [`gdu`](https://github.com/dundee/gdu)
  * [`godu`](https://github.com/viktomas/godu)
* **GUI:**
  * [GNOME's Disk Usage Analyzer, a.k.a. `baobab`](https://wiki.gnome.org/action/show/Apps/DiskUsageAnalyzer)
  * Filelight
  
[petgraph]: https://crates.io/crates/petgraph
[rustup]: https://rustup.rs/
[jwalk]: https://crates.io/crates/jwalk
[termion]: https://crates.io/crates/termion
[tui]: https://github.com/fdehau/tui-rs
