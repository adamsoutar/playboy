# Building Playboy from source

**Please note:** If you just want to play Gameboy games on your Playdate, _you don't need to do this!_

These steps are for folks who want to work directly on the code for Playboy
itself.

Playdate isn't exactly a Tier 1 platform for Rust :)

As a result, the build is a _little_ bit trickier than usual, but you can install
Playboy by following these steps.

### Prerequisites

You'll need

- The [Playdate SDK](https://play.date/dev/)
- [Rustup](https://rustup.rs)
- Nightly Rust (run`rustup default nightly`)
- [Crank](https://github.com/rtsuk/crank), the actual tool for compiling Rust for Playdate

### Running

```bash
git clone https://github.com/adamsoutar/playboy
cd playboy
git submodule init
git submodule update
```

> Next, you'll need to place a Gameboy game in the `playboy` folder as `rom.gb`.
> The Playboy repo doesn't contain any copyrighted Nintendo software or any built-in ROMs.

```bash
cp ~/MyPersonalRomCollection/Tetris.gb ./rom.gb
```

> Now we can finally build & run the code:

```bash
crank run --release
```

This will launch the Playdate Simulator and boot right in to your favourite
Gameboy game!
