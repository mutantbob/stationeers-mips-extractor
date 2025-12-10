This project has two tools

`extract-mips` can extract MIPS programs from the IC10s in a stationeers save file.

`guess-diff` will compare the extracted MIPS programs to `.mips` files in a different directory to guess which source each one is most similar to.

```
cargo run --bin extract-mips /tmp/world.xml /tmp/mips/
cargo run --bin guess-diff ~/src/stationeers-cookbook/ /tmp/mips/
```

You can find your Stationeers save data in `c:\Users\*\Documents\My Games\Stationeers\saves\*\world.xml`
