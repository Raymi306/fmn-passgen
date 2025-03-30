#!/bin/sh

PKGID_PREFIX="$(cargo pkgid | cut -f 1 -d ':')"
if [ "$PKGID_PREFIX" != "path+file" ]; then
    echo "Unexpected output for `cargo pkgid`, probably need to edit the script again"
    exit 1
fi
VERSION="$(cargo pkgid | cut -f 2 -d '#')"
COMMIT="$(git rev-parse --verify HEAD)"
VERIFY_EXIT=$?

if [ $VERIFY_EXIT -ne 0 ]; then
    echo "git rev-parse --verify HEAD failed"
    exit $VERIFY_EXIT
fi

SHORTCOMMIT="$(git rev-parse --short HEAD)"

cargo build --profile release
cargo build --profile small
cargo build --profile release --features gui --bin fmn-passgen-gui
cargo build --profile small --features gui --bin fmn-passgen-gui

TMPFILE="$(basename $0)".md.tmp
hyperfine --export-markdown $TMPFILE --warmup 3 'target/release/fmn-passgen -c 255 > /dev/null' 'target/small/fmn-passgen -c 255 > /dev/null'

echo "# $VERSION Benchmarks - [$SHORTCOMMIT](https://github.com/Raymi306/xkcd-password-gen/tree/$COMMIT)\n" > benchmarks/README.md
echo "## hyperfine\n" >> benchmarks/README.md

cat $TMPFILE >> benchmarks/README.md
rm $TMPFILE

echo "\n## Binary Sizes\n" >> benchmarks/README.md
echo "- $(stat -c %s target/release/fmn-passgen | numfmt --to=iec) cli, release" >> benchmarks/README.md
echo "- $(stat -c %s target/small/fmn-passgen | numfmt --to=iec) cli, small" >> benchmarks/README.md
echo "- $(stat -c %s target/release/fmn-passgen-gui | numfmt --to=iec) gui, release" >> benchmarks/README.md
echo "- $(stat -c %s target/small/fmn-passgen-gui | numfmt --to=iec) gui, small" >> benchmarks/README.md
echo "\n## Wordlist Sizes\n" >> benchmarks/README.md
echo "- $(stat -c %s wordlists/eff_large_wordlist.txt | numfmt --to=iec) eff_large_wordlist.txt" >> benchmarks/README.md

cp benchmarks/README.md benchmarks/$VERSION-$SHORTCOMMIT.md
