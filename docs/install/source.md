## Installing from source

If your system isn't covered by the pre-built packages then you can build it
for yourself.  WezTerm should run on any modern unix as well as Windows 10 and
macOS.

* Install `rustup` to get the `rust` compiler installed on your system.
  [Install rustup](https://rust-lang.org/tools/install/).
* Rust version 1.71 or later is required
* Build in release mode: `cargo build --release`
* Run it via either `cargo run --release --bin wezterm` or `target/release/wezterm`

You will need a collection of support libraries; the [`get-deps`](https://github.com/wezterm/wezterm/blob/main/get-deps) script will
attempt to install them for you.  If it doesn't know about your system,
[please contribute instructions!](https://github.com/wezterm/wezterm/blob/main/CONTRIBUTING.md).

If you don't plan to submit a pull request to the wezterm repo, you can
download a smaller source tarball using these steps:

```console
$ curl https://sh.rustup.rs -sSf | sh -s
$ curl -LO {{ src_stable }}
$ tar -xzf {{ src_stable_asset }}
$ cd {{ src_stable_dir }}
$ ./get-deps
$ cargo build --release
$ cargo run --release --bin wezterm -- start
```

Alternatively, use the full git repo:

```console
$ curl https://sh.rustup.rs -sSf | sh -s
$ git clone --depth=1 --branch=main --recursive https://github.com/wezterm/wezterm.git
$ cd wezterm
$ git submodule update --init --recursive
$ ./get-deps
$ cargo build --release
$ cargo run --release --bin wezterm -- start
```

**If you get an error about zlib then you most likely didn't initialize the submodules;
take a closer look at the instructions!**

### Building a macOS app bundle

From a Git checkout, run:

```console
$ ./ci/build-macos.sh
```

Run the same command after pulling changes or editing the source. It updates
submodules to the revisions recorded in your checkout and incrementally builds
the current source in release mode for your Mac's architecture (Apple Silicon
or Intel, not a universal binary). You can invoke the script from any directory.

Prerequisites: Rust/Cargo, CMake, pkg-config, and Xcode Command Line Tools. With
Homebrew, install the build tools using `brew install rust cmake pkg-config`;
install Apple's tools with `xcode-select --install` if needed. The build script
does not install dependencies itself.

Outputs:

* `target/macos/WezTerm.app`
* `target/macos/WezTerm-macos.zip`

The bundle includes the GUI, CLI, mux server, shell integration, completions,
and terminfo. It uses bundled libgit2 rather than Homebrew's shared library and
checks that the executables depend only on macOS system libraries. The app is
ad-hoc signed for local use, **not Apple-notarized**.

Each successful run replaces the previous generated app and ZIP. Quit a running
copy of that generated app before rebuilding. Your installed app and
`wezterm.lua` are not changed; copy the generated app to Applications yourself
if desired. To launch it directly from the checkout:

```console
$ open target/macos/WezTerm.app
```

For OSC 52 clipboard reads, see
[`allow_osc52_clipboard_read`](../config/lua/config/allow_osc52_clipboard_read.md).

### Building without Wayland support on Unix systems

By default, support for both X11 and Wayland is included on Unix systems.
If your distribution has X11 but not Wayland, then you can build WezTerm without
Wayland support by changing the `build` invocation:

```console
$ cargo build --release --no-default-features --features vendored-fonts
```

Building without X11 is not supported.

### Building on Windows

When installing Rust, you must use select the MSVC version of Rust. It is the
only supported way to build wezterm.

On Windows, instead of using `get-deps`, the only other dependency that you need is
[Strawberry Perl](https://strawberryperl.com). You must ensure that you have
your `PATH` environment set up to find that particular `perl.exe` ahead of any
other perl that you may have installed on your system. This particular version
of perl is required to build openssl on Windows.

```console
$ set PATH=c:\Strawberry\perl\bin;%PATH%
```

