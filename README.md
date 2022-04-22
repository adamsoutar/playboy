![Playboy logo](./assets/Logo-GitHub-Grey.svg#gh-dark-mode-only)
![Playboy logo](./assets/Logo-Black.svg#gh-light-mode-only)

Playboy is a Nintendo Gameboy emulator for the [Panic Playdate](https://play.date)!

<table>
  <tr>
    <td><img src="./assets/mario.gif" /></td>
    <td><img src="./assets/tetris.gif" /></td>
  </tr>
</table>

## Bring your own games

The first time you start Playboy, you'll see "**_No game ROM found_**".

See [these steps](./docs/adding-roms.md) for downloading/playing whichever games
you want.

## Controls

**Left**, **Right**, **Up**, **Down**, **A** and **B** are exactly what you'd
expect.

**Start** and **Select** are interesting, because the Playdate doesn't have
enough buttons!

Turn the crank clockwise to press **Start**, and counter-clockwise to press
**Select**.

## The core

Playboy is powered by my Rust Gameboy Emulator project [gbrs](https://github.com/adamsoutar/gbrs).

_If you fancy a challenge_, I'd love to see people try and port it to more systems!

For the platform-specific side of things, it uses the awesome [crankstart](https://github.com/rtsuk/crankstart) bindings by Rob Tsuk.

---

<h6 align="center">By Adam Soutar</h6>
