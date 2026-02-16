# Mithrandir
This is a simple small package manager called Mithrandir.
It's nothing much, but i think it has something special. It is very customizable.

On the recipe.json you can choose the programs the package manager will execute to install certain packages. This allows the package manager to install binaries, compile from source ,install other files like .deb,.rpm etc.

But this works in theory. It may not be very practical, but it is very usefull if you want to learn Linux.
If you are a beginner don't install Mithrandir, just like Gandalf, Mithrandir is powerful but can be defeated by a Balrog on your PC.

I hope you like it.

Anonymous King.

## Installation

Use the installer to build and install `mtr` system-wide:

```bash
./install.sh
```

This installs:
- executable: `/usr/local/bin/mtr`
- runtime files: `/usr/local/bin/src/*`
- package store: `/usr/local/bin/mtr/store`

After that, run commands like:

```bash
mtr go
```
