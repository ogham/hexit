Hexit
=====

Hexit is a tiny programming language that deals with bytes. It’s useful for writing binary formats and network packets without the tedium of forgetting which byte means what, or having to stumble around a hex editor. I like to think of it as “Markdown for binary”.

As an interpreted programming language, Hexit works like other interpreters: give it a file to run, or it’ll expect a program on stdin.

    hexit [OPTIONS] PROGRAM.hexit


Full Example
------------

Here’s an example of a Hexit program, describing a BGP packet:

    # Header
    Marker:    x12(FF)
    Length:    be32[29]
    Type:      BGP_OPEN
    Version:   04
    ASN:       be32[12345]
    Hold time: be32[180]

    # Data
    Identifier: [192.168.0.1]
    Optional params: 00

And here’s what it emits:

    FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF001D0104303900B4C0A8000100

- The text after a `#` is a comment. Anything on a line before a colon is a comment, too (reverse comments!)
- Bytes are read in as pairs of hex characters. Everything from `00` to `FF` just outputs itself. You don’t need to prefix anything with `0x`. These _must_ be paired: `0` on its own is a syntax error.
- Decimal numbers are enclosed in square brackets. `FF` and `[255]` are equivalent.
- Function calls use parentheses. `x12(FF)` applies the function `x12` to the byte `FF`. That function repeats the byte twelve times. (There are others like it.) You don’t need commas to separate arguments.
- Decimal numbers larger than 255 aren’t accepted by themselves. You’ll need to specify a size and endianness to output them. This is done by functions such as `be32` (big-endian, 4 bytes wide) or `le16` (little, 2 bytes).
- Passing one decimal number to a function is so common, you can write `be32[180]` instead of `be32([180])`.
- IPv4 addresses resolve to four bytes.


Customising the output
----------------------

By default, Hexit prints out its bytes using two hex characters each: an input consisting of two characters `6` and `B` will be written using those same two characters, `6` and `B`.

If you’re actually sending data somewhere, though, you might prefer it to output the _byte_ 0x6B (which is 107 in decimal, or `k` in ASCII). You can do this with `--raw`. Alternatively you can pipe the output through `xxd -r -p`.

If you want the output to be _more_ human-readable, you can use these options to make it a bit prettier:

- **--prefix**: String to print _before_ a pair of hex characters.
- **--suffix**: String to print _after_ a pair of hex characters.
- **--separator**: String to print _between_ successive pairs of hex characters.
- **--lowercase**: If you like your letters minuscule.

A nice example is `--separator=":"` for colon-separated bytes. Or `--prefix="0x" --separator=" "` if you need another program to read the bytes back in.
<!-- I know you don’t strictly need the quotes! -->


Checking the output
-------------------

You can verify that the output seems correct with one of these options:

- **--verify-length**: If you know the exact length the output should be, you can tell Hexit to fail if it’s not.
- **--verify-boundary**: Similarly, if you don’t know the length, but _do_ know that it should be a multiple of a power of two, you can check that it falls on the correct byte boundary.


What it doesn’t do
------------------

- It doesn’t send data over a network. Use `netcat` or `nping` for that.
- It doesn’t make streams of data human-readable. Use `xxd` for that.
- It doesn’t parse streams of data back into their structures. Use Wireshark or `file` for that.


Licence
-------

Hexit is dual-licenced under the [CC0](https://creativecommons.org/share-your-work/public-domain/cc0/) and [MIT](https://opensource.org/licenses/MIT) licences. For more information, see the [Why the licence?](https://github.com/ogham/hexit/wiki/Why-the-licence%3F) wiki page.
