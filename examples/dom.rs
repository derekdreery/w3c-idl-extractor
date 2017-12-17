#[macro_use] extern crate failure_derive;
extern crate failure;
extern crate ansi_term;
extern crate w3c_idl_extractor;

use std::io;
use std::fs;

use failure::Error;

#[derive(Fail, Debug)]
enum ExampleError {
    #[fail(display = "cannot open `DOM.html`, make sure the cwd is correct")]
    FileError(#[cause] io::Error)
}

fn run() -> Result<(), Error> {
    use ansi_term::Colour::Green;
    let mut html_doc = fs::File::open("./examples/DOM.html")
        .map_err(ExampleError::FileError)?;
    let idl = w3c_idl_extractor::extract_idl(&mut html_doc)?;
    for (idx, frag) in idl.iter().enumerate() {
        println!("{}\n{}\n", Green.paint(format!("IDL fragment {}", idx)), frag);
    }
    Ok(())
}

fn main() {
    use ansi_term::Colour::{Red, Yellow};
    if let Err(e) = run() {
        let mut causes = e.causes();
        println!("{}: {}\n", Red.paint("Error"), causes.next().unwrap());
        for cause in causes {
            println!("  {}: {}\n", Yellow.paint("caused by"), cause);
        }
    }
}
