# tui_document Goal:
Bring the rope data structure to a Ratatui widget. This crate wraps the ['ropey'](https://crates.io/crates/ropey) crate. 
The rope data structure is a binary tree used for efficiently manipulating long strings. The author of
the ropey crate has a great paper on ['Ropey's Design'](https://github.com/cessen/ropey/blob/master/design/design.md). his specific implementation of this data structure.

# When should I use tui_document:
The underlying ropey crate allocates space in kilobytes. Small text documents don't need a rope.
This crate is for things like text editors or large log files. Medium to large documents that
require frequent edits or efficient search functionality.

# This is information quoted from the ropey crate documentation.
- On a recent mobile i7 Intel CPU, Ropey performed over 1.8 million small
  incoherent insertions per second while building up a text roughly 100 MB
  large.  Coherent insertions (i.e. all near the same place in the text) are
  even faster, doing the same task at over 3.3 million insertions per
  second.
- Freshly loading a file from disk only incurs about 10% memory overhead.  For
  example, a 100 MB text file will occupy about 110 MB of memory when loaded
  by Ropey.
- Cloning ropes is _extremely_ cheap.  Rope clones share data, so an initial
  clone only takes 8 bytes of memory.  After that, memory usage will grow
  incrementally as the clones diverge due to edits.
  
# Disclaimer:
 Most of the good code and documentation in this crate has been taken directly from Ropey source code.
 My goal in this project is to produce a widget that wraps a great crate and bring it to the Ratitui community. Rather than
 mark every function that has taken influence or been copied from the Ropey source code or mark each piece of
 documentation that was copied, modified, editied, etc after copying from Ropey source code.

All credit goes to [cessen](https://github.com/cessen/ropey) and the [ropey crate](https://crates.io/crates/ropey) crate.
