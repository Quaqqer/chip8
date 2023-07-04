# chip8

<p align="center">
  <img src="https://github.com/Quaqqer/chip8/blob/master/res/chip8.png">
</p>

This is my implementation of a CHIP-8 emulator.
It was a fun bite-sized project I finished in a day to introduce me to
emulators.

I've tried the emulator with some games and it seems to behave correctly.
It passes the [chip8-test-suite](https://github.com/Timendus/chip8-test-suite)
by Timendus and I've tried it with the Tetris rom as well as Breakout.

Audio has not been implemented. Neither does it implement the drawing quirk of
the original chip8 being limited in the amount of sprite draws.

# Installing

To install simply run the commands

```bash
$ git clone https://github.com/Quaqqer/chip8.git
$ cd chip8
$ cargo install --path .
```

and to run it run the commands

```bash
$ chip8 <path/to/rom>
```
