# Brainfreeze

A program to hide [Brainf*ck](https://esolangs.org/wiki/Brainfuck) program inside images

![Example Image](.github/example.png)
> This photo has the program that prints out "zetsuboii"
> hidden inside

## Usage

```bash
# Store a Brainf*ck program inside a PNG
# For now, only PNG format is supported
$ brainfreeze inject original.png hello.bfk -o hidden.png

# Run the program inside the PNG
$ brainfreeze run hidden.png
```

## Wait, how does it even work?

Glad you asked.

Brainf*ck is a dead simple language that only has
9 tokens, including EOF. This makes hiding it quite simple.

In order to encode a brainf*ck token to image, I used differences
of RGBA values between each pixel. It's uncommon to have the same
difference between two pixels, and inside `brainfreeze` a token is
represented by a constant color change.

As an optimization, tokens that can stack (`>`, `<`, `+` and `-`) 
are represented by a single token followed by empty pixels 
accounting for the stack amount.