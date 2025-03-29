# ![Forget-me-not Flower Icon](icon.png) fmn-passgen ![cargo-audit](https://github.com/Raymi306/xkcd-password-gen/actions/workflows/audit.yml/badge.svg)
*Forget-me-not Password Generator*

Every password you use should be randomly generated and secured by a password manager.

However, you may find yourself in need of a password to protect the password manager.

`fmn-passgen` can generate strong passwords that can be easier to type and remember than a shorter entirely random password.

The default settings require memorization of 2 random symbols, 2 digits, and 4 words with a simple mixture of capital and lowercase letters:

`:unending=RUST=stumble=OUTSKIRTS=94:`

This should require less time to memorize than an equivalently strong password of random characters, while also being faster and less error-prone to enter for individuals comfortable with typing.
These settings can be easily adjusted to change the memorability and strength of the generated passwords.

Note that if you are a slow typer and you do not need the password to be memorable, a standard randomly generated password can provide equivalent protection with less overall length.

## Inspiration
- [xkcd 936](https://xkcd.com/936/)
- https://www.xkpasswd.net/

## Resources

Wordlist courtesy of the Electronic Frontier Foundation
- https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt

Logo by iiintangible
- https://tenderlo.in

## Examples

```
Usage: fmn-passgen [options]

Options:
    -h, --help
    -c, --count NUM, default=1
                        how many passwords to make
    -w, --word-count NUM, default=4
                        number of words
    -m, --word-min-length NUM, default=3
                        minimum length of a chosen word
    -M, --word-max-length NUM, default=11
                        maximum length of a chosen word
    -W, --word-transformation TYPE, default=alternating-lower-upper
                        transformation to apply to the selected words
    -b, --digits-before NUM, default=0
                        number of digits to prepend
    -a, --digits-after NUM, default=2
                        number of digits to append
    -T, --padding-type TYPE, default=fixed
                        how to apply padding
    -l, --padding-length NUM, default=1 for fixed, 42 for adaptive
                        how much to pad
    -p, --padding-characters CHOICES, default="!@$%^&*-_+=:|~?/.;"
                        list of characters to choose from
    -s, --separators CHOICES, default="!@$%^&*-_+=:|~?/.;"
                        list of characters to choose from
    -r, --rng TYPE, default=os-rng
                        method of random number generation

types are case insensitive

WORD TRANSFORMATIONS:
    none
    lower                   (correct horse battery staple)
    upper                   (CORRECT HORSE BATTERY STAPLE)
    capitalize-first        (Correct Horse Battery Staple)
    capitalize-last         (correcT horsE batterY staplE)
    capitalize-not-first    (cORRECT hORSE bATTERY sTAPLE)
    alternating-lower-upper (correct HORSE battery STAPLE)
    alternating-upper-lower (CORRECT horse BATTERY staple)
    random-upper-lower      (correct HORSE battery staple)

PADDING TYPES:
    none
    fixed    (add padding-length padding-characters to front and back)
    adaptive (if unpadded password is less than padding-length, append padding-characters to meet length)

RNG TYPES:
    os-rng (the system's native secure RNG)
    csprng (a reasonably secure userspace RNG)
```

```
$ fmn-passgen -c 3
?emblem|DOORMAN|luckiness|BROADNESS|19?
=obsessed@CIRCULATE@epidemic@SPOTTED@90=
!blouse|CHANNEL|venture|XEROX|79!
```

![428259203-99956de7-2685-4c55-8ab7-c343fca2b88a](https://github.com/user-attachments/assets/6ec1453a-6b93-44cd-b1b8-7c8747fb21b1)

## Features

- gui
  - enable dependencies to support building a GUI frontend.

## Installation

Download the appropriate release for your operating system, or build from scratch.
The checksum can be used to easily validate that the archive has not been corrupted or tampered with.

On most Linux systems, you can validate the archive by running `sha256sum -c $CHECKSUM_FILE` when the checksum and the archive are in the same directory.

### Building from Scratch

1. https://www.rust-lang.org/tools/install
2. `git clone` or otherwise acquire the source code for this project.
3. `cargo build --profile small`

## Core Dependencies

- rand = "0.9" [(docs)](https://docs.rs/rand/latest/rand/)
- rand_core = "0.9" [(docs)](https://docs.rs/rand_core/latest/rand_core/)
- getopts = "0.2" [(docs)](https://docs.rs/getopts/latest/getopts/)

## GUI Dependencies

- eframe = "0.31" [(eframe docs)](https://docs.rs/eframe/latest/eframe/) [(egui docs)](https://docs.rs/egui/latest/egui/index.html)
- egui_extras = "0.31" [(docs)](https://docs.rs/egui_extras/latest/egui_extras/)
- ~~image = "0.25"~~ - *only for adding png support to egui_extra to load the icon*

## [Benchmarks](benchmarks)

### Tooling

[hyperfine](https://github.com/sharkdp/hyperfine)

## TODO

### Features

- config files
- presets
- statistics (entropy)
- custom wordlists
- feature flag to not include default wordlist
- short/long help or manpage
- explore reducing binary sizes more
- symmetrical padding option, eg `*#$[PASSWORD]$#*`
- additional CSPRNG options under feature flags?
- dice RNG feature flag

### Housekeeping

- test coverage
- more macro magic
- make own UnwrapErr for TryRngCore

## Checklist Before Release

- [ ] README.md up-to-date?
- [ ] Cargo.toml package version bumped?
- [ ] `./run_bench.sh`?
