This is a protoype that provides binary-to-text encoding and decoding
using Crockford Base 32, [G60](https://github.com/galenhuntington/g60),
and [G86](https://github.com/galenhuntington/g86).  It is meant as a
general platform for encoding/decoding as well as an implementation
of each of those.

It can be invoked as `genc` with an option specifying the encoding,
or if invoked as `g32`, `g60`, or `g86`, it will use the corresponding
encoding.

