# 1.2.0 Benchmarks - [9594493](https://github.com/Raymi306/xkcd-password-gen/tree/9594493807864fdeab4b88deb598c94df1454cfb)

## hyperfine

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/fmn-passgen -c 255 > /dev/null` | 37.8 ± 10.3 | 25.6 | 60.9 | 1.02 ± 0.40 |
| `target/small/fmn-passgen -c 255 > /dev/null` | 36.9 ± 10.1 | 24.5 | 63.1 | 1.00 |

## Binary Sizes

- 815K cli, release
- 771K cli, small
- 11M gui, release
- 8.0M gui, small

## Wordlist Sizes

- 61K eff_large_wordlist.txt
