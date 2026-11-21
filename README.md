# imp-rs

Reader and writer for Warcraft III import lists, `war3map.imp`: the
files World Editor's Import Manager added to a map, by path. A library,
a small CLI and a text form; the [wc3-rs](../wc3-rs) editor opens `.imp`
files as that text.

Reading and writing are exact, and so is the text form, on every map
of the game and the corpus (14 files, 876 imports).

## Format

| type | field |
|---|---|
| u32 | version: 0 Reign of Chaos, 1 The Frozen Throne |
| u32 | the number of files |
| | then each file: |
| u8 | (1) flags |
| string | its path |

**Flags.** `0x01` is a custom path: the file is at its path as stored
(`Airlock.blp` at the map's top, `Armor\ArmorCrab.mdx`). Without it, the
file is at `war3mapImported\` and the path's file name
(`Import::path_in_map`); version 0 has no flags, and its files are
there too. The maps at hand store `13` (`0x01 0x04 0x08`), `8` and `0`;
`0x02` and up aren't known well (one description has `0x02` drop the
entry when read), and are kept as they are.

## Text

```ini
[imp]
version=1
file=13,"Armor\ArmorCrab.mdx"
file=8,"Acolyte01.mp3"
```

A `file=` line per file, in order: its flags (version 1) and its path,
quoted. Where the file is in the map and the flags' names are for an
editor to show: the wc3-rs editor does, as inlay hints.

## CLI

```sh
imp-rs war3map.imp          # read and write back: a check
imp-rs --list war3map.imp   # flags, path, path in the map, tab-separated
```

## Tests

```sh
cargo test
IMP_CORPUS=<folder of map folders> cargo test --release corpus -- --ignored
```
