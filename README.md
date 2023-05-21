# Goomba

Goomba is a GameBoy emulator written in Rust.

Try it at <https://teskje.github.io/goomba>!

## Usage

Goomba can run natively on desktop platforms and inside the browser.

### Desktop

You can run Goomba natively using the `cargo run` command.
It expects you to specify the path to a ROM (`.gb`) file:

```
$ cargo run --release -- roms/zelda.gb 
```

This opens a window showing the GameBoy display.

<img width="400" alt="Goomba desktop UI" src="https://github.com/teskje/goomba/assets/4521314/d20d036f-6d5b-4e06-a99b-7c7bebf5889d">

Controls are:

| Joypad Button | Keyboard Key |
|---------------|--------------|
| Up            | Arrow Up     |
| Down          | Arrow Down   |
| Left          | Arrow Left   |
| Right         | Arrow Right  |
| A             | x            |
| B             | z            |
| Start         | Enter        |
| Select        | Backspace    |

Upon quitting the emulator, a dialog opens that allows you to save the current cartridge RAM.
Saving the RAM is necessary to be able to continue playing from in-game save points.
To do so, specify the `.gb-ram` file as an additional argument to the `cargo run` command:

```
$ cargo run --release -- roms/zelda.gb --ram-path roms/zelda.gb-ram
```

You can also save a snapshot of the game by pressing `ctrl-s`.
This opens a dialog that lets you save a `.gb-save` file, which can be used to later load the same game state again:

```
$ cargo run --release -- roms/zelda.gb-save
```

Note that `.gb-save` files are generally not compatible across different code versions.

### Browser

To run Goomba in the browser, you need to first install the web bundler [trunk](https://trunkrs.dev).
Then run it to serve Goomba locally:

```
$ trunk serve --release web/index.html
[...]
2023-05-21T14:03:06.846895Z  INFO ✅ success
2023-05-21T14:03:06.847740Z  INFO 📡 serving static assets at -> /
2023-05-21T14:03:06.847797Z  INFO 📡 server listening at http://127.0.0.1:8080
```

At <http://127.0.0.1:8080> you are now presented with Goomba's web UI.

<img width="400" alt="Goomba web UI" src="https://github.com/teskje/goomba/assets/4521314/3846b787-74b3-44ae-92fc-d4154aac9f96">

Use the leftmost menu button to load a ROM to play.
The keyboard controls are the same as for desktop.
On mobile devices that don't have a keyboard, you can control the game using the joypad instead.

You can save the cartridge RAM using the "save" button in the menu.
The resulting `.gb-ram` file can be provided when you next load the game through the "load" button.
Be sure the select *both* the `.gb` and the `.gb-ram` file in the file dialog.

## TODOs

* [ ] Audio emulation
* [ ] GameBoy Color support
