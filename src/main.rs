use std::{
    borrow::Cow,
    env,
    fs::{self, File},
    io::{self, BufWriter, ErrorKind, Write},
    path::Path,
};

fn main() {
    match run() {
        Ok(output) => println!("{output}"),
        Err(err) => eprintln!("{err}"),
    }
}

fn run() -> Result<String, Cow<'static, str>> {
    let mut args = env::args_os();

    args.next(); // SKip the program name

    if let Some(raw_path) = args.next() {
        if let Some(path) = raw_path.to_str() {
            let path = Path::new(path);

            if let Some(extension) = path.extension() {
                if extension == "rs" {
                    let input = match fs::read_to_string(path) {
                        Ok(input) => input,
                        Err(err) => match err.kind() {
                            ErrorKind::NotFound => Err("File not found"),
                            _ => Err("Error reading file"),
                        }?,
                    };

                    let file_name = path.file_name().unwrap().to_str().unwrap();

                    let path = path.with_file_name(&format!("shrunken-{file_name}"));

                    let file = match File::create(&path) {
                        Ok(file) => file,
                        Err(_) => {
                            return Err(
                                format!("Error creating {file_name} to write output to").into()
                            )
                        }
                    };

                    let mut writer = BufWriter::new(file);

                    match shrink(input, &mut writer) {
                        Ok(()) => {}
                        Err(_) => return Err("Error writing file".into()),
                    }

                    Ok(format!("Output written to `{}`", path.to_str().unwrap()))
                } else {
                    Err("Filename does not have an `.rs` extension".into())
                }
            } else {
                Err("Filename does not have an extension".into())
            }
        } else {
            Err("Path is not valid UTF-8".into())
        }
    } else {
        Err("No path given.\nExample: `shrink-rs hello.rs` will output a compressed `shrunken-hello.rs` with comments removed.".into())
    }
}

fn shrink(input: String, writer: &mut impl Write) -> io::Result<()> {
    use rustc_lexer::{tokenize, Token, TokenKind};

    fn layout(
        tokens: impl Iterator<Item = Token>,
        input: &str,
        writer: &mut impl Write,
    ) -> io::Result<()> {
        use TokenKind::*;
        let mut last_ident_like = false;
        let mut i = 0;

        for Token { kind, len } in tokens {
            let output = &input[i..i + len];

            match kind {
                Whitespace | LineComment { .. } | BlockComment { .. } => {}
                Ident | RawIdent | Literal { .. } | Lifetime { .. } if last_ident_like => {
                    write!(writer, " {output}")?;
                }
                Ident | RawIdent | Literal { .. } | Lifetime { .. } => {
                    last_ident_like = true;
                    write!(writer, "{output}")?;
                }
                _ => {
                    last_ident_like = false;
                    write!(writer, "{output}")?;
                }
            };

            i += len;
        }
        Ok(())
    }

    let tokens = tokenize(&input);
    layout(tokens, &input, writer)
}

mod tests {
    #[test]
    fn fizz() {
        use super::*;

        let input = fs::read_to_string("examples/fizz.rs").unwrap();
        let mut output = Vec::new();
        shrink(input, &mut output).unwrap();

        let expectation = fs::read("examples/shrunken-fizz.rs").unwrap();

        assert_eq!(output, expectation);
    }
}
