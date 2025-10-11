pub mod base;
pub mod g32;
pub mod g60;
pub mod g86;
pub mod wrap;
pub mod test;

use std::io::{stdout,Write};
use std::path::PathBuf;

use base::Encoding;
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
        if let Err(e) =
            T::decode_valid(
                &mut std::io::stdin(),
                &mut stdout,
                base::FilterType::Whitespace,
                ) {
            eprintln!("Error: {}", e);
        }
    } else {
        T::encode(
            &mut std::io::stdin(),
            &mut WrapWidth::new(stdout, width),
            ).unwrap();
        stdout.write(b"\n").unwrap();
        stdout.flush().unwrap();
    }
}

fn dispatch_enc<T: Encoding>() {
    per_enc::<T>(O::parse());
}

fn dispatch_all() {
    match C::parse() {
        C { encoding, rest } => {
            if encoding.g32 { per_enc::<G32>(rest); }
            else if encoding.g60 { per_enc::<G60>(rest); }
            else if encoding.g86 { per_enc::<G86>(rest); }
        }
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

