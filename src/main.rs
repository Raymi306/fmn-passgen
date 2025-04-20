//
//  A memorable password generator inspired by xkcd comic 936.
//  Copyright (C) 2025  Andrew Langmeier
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published
//  by the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.txt>.
//
//! Create memorable passwords.
//!
//! Use custom configurations, or roll with the defaults.
use std::env;
use std::process::ExitCode;

use getopts::Options;
use rand::rngs::OsRng;

use fmn_passgen::password_maker::PasswordMaker;
use fmn_passgen::parser::parse;

/// The entrypoint.
///
/// Here, we define the program's CLI arguments.
/// We use the [`getopts` library](https://docs.rs/getopts/latest/getopts/) to accomplish this.
/// We check which arguments the user passed in.
/// Finally, we generate passwords using the specified arguments.
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "");
    opts.optopt(
        "f",
        "format",
        "the format string describing the desired password",
        "STR",
    );
    opts.optopt(
        "c",
        "count",
        "how many passwords to make",
        "NUM, default=1",
    );
    let matches = match opts.parse(&args[1..]) {
        Ok(v) => v,
        Err(failure) => {
            eprintln!("{failure}");
            return ExitCode::FAILURE;
        }
    };
    if matches.opt_present("h") || !matches.free.is_empty() {
        let brief = format!("Usage: {program_name} [options]");
        println!("{}", opts.usage(&brief));
        return ExitCode::SUCCESS;
    }

    let count = match matches.opt_get_default("c", 1) {
        Ok(v) => v,
        Err(failure) => {
            eprintln!("{failure}");
            return ExitCode::FAILURE;
        }
    };

    let format_str = matches.opt_str("f").unwrap_or(
        "{(word|lower)(1@symbol)(word|upper)(1@symbol)}!2(digit)!4(symbol)".to_owned()
    );

    let expressions = match parse(&format_str) {
        Ok(v) => v.1,
        Err(failure) => {
            eprintln!("{failure}");
            return ExitCode::FAILURE;
        }
    };

    let result = PasswordMaker::<OsRng>::default().make_passwords(&expressions, count);
    for password in result {
        println!("{password}");
    }
    ExitCode::SUCCESS
}
