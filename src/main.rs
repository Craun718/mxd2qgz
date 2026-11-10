use std::{env, io, path::Path};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let command = args.next();
    let path = args.next();
    if command.as_deref() != Some(std::ffi::OsStr::new("inspect"))
        || path.is_none()
        || args.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz inspect <file.mxd>",
        ));
    }
    let path = Path::new(path.as_ref().unwrap());
    let document = cfb::open(path)?;
    println!("Document: {}", path.display());
    for entry in document.walk().filter(|entry| entry.is_stream()) {
        println!("{:>10}  {}", entry.len(), entry.path().display());
    }
    Ok(())
}
