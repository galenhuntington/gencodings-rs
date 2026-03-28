pub mod base;
pub mod g32;
pub mod g60;
pub mod g86;
pub mod wrap;
#[cfg(test)]
pub mod test;

use std::io::{stdout,Write,copy};
use std::path::PathBuf;

use base::{Encoding,DecMode};
use wrap::WrapWidth;
use g32::G32;
use g60::G60;
use g86::G86;
use clap::{Parser,Args};

#[derive(Debug,Parser)]
struct C {
    #[clap(flatten)]
    encoding: EncodingOpts,
    #[clap(flatten)]
    rest: O,
}

#[derive(Debug,Args)]
#[group(required = true, multiple = false)]
pub struct EncodingOpts {
    #[clap(long = "32", short = '3')]
    /// Crockford Base 32 encoding
    g32: bool,
    #[clap(long = "60", short = '6')]
    /// G60 encoding
    g60: bool,
    #[clap(long = "86", short = '8')]
    /// G86 encoding
    g86: bool,
}

#[derive(Debug,Parser)]
pub struct O {
    #[clap(long, short)]
    /// Decode
    decode: bool,
    #[clap(long, short, default_value_t = 72)]
    /// Line width; 0 for no wrapping
    width: usize,
}

fn per_enc<T: Encoding>(O { width, decode }: O) {
    let mut stdout = &stdout();
    if decode {
        if let Err(e) = copy(
                    &mut std::io::stdin(),
                    &mut T::new_decoder(&mut stdout, DecMode::Whitespace),
                ) {
            eprintln!("Error: {}", e);
        }
    } else {
        copy(
            &mut std::io::stdin(),
            &mut T::new_encoder(&mut WrapWidth::new(stdout, width)),
            ).unwrap();
        stdout.write_all(b"\n").unwrap();
        stdout.flush().unwrap();
    }
}

fn dispatch_enc<T: Encoding>() {
    per_enc::<T>(O::parse());
}

fn dispatch_all() {
    let C { encoding, rest } = C::parse();
    match () {
        _ if encoding.g32 => per_enc::<G32>(rest),
        _ if encoding.g60 => per_enc::<G60>(rest),
        _ if encoding.g86 => per_enc::<G86>(rest),
        _ => panic!(),
    }
}

fn main() {
    let ex = std::env::args_os().next()
        .and_then(|p| PathBuf::from(p).file_stem().map(|s| s.to_owned()));
    match ex {
        Some(f) if f == "g32" => dispatch_enc::<G32>(),
        Some(f) if f == "g60" => dispatch_enc::<G60>(),
        Some(f) if f == "g86" => dispatch_enc::<G86>(),
        _                     => dispatch_all(),
    }
}

