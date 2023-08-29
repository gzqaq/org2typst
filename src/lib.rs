use clap::Parser;
use regex::{Captures, Regex};
use std::fs::{canonicalize, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    org_file: PathBuf,
    typst_file: Option<PathBuf>,
}

pub fn run(mut config: Config) -> Result<(), &'static str> {
    config.org_file = match canonicalize(&config.org_file) {
        Ok(path) => path,
        Err(_) => return Err("Unable to canonicalize path to org file"),
    };

    let org_file = match File::open(&config.org_file) {
        Ok(file) => file,
        Err(_) => return Err("Failed to open org file"),
    };
    let mut org_file = BufReader::new(org_file);
    let mut org_contents = String::new();
    org_file
        .read_to_string(&mut org_contents)
        .expect("Unable to read org file");

    let patterns = Regex::new("#\\+title:\\s([\\w-]+)\\s#\\+author:\\s([\\w ]+)|\\n(\\*+)|(#\\+begin_src\\s|#\\+end_src)|(:PROPERTIES:\\s:ID:\\s+[\\w-]*\\s:END:)|(#\\+.+\\s)|/([\\w ]+)/|\\[\\[id:.+\\]\\[([\\w-]+)\\]\\]|``([\\w\\s]+)''|\\[cite:(@\\w+)\\]").expect("Unable to create regex pattern");
    let result = patterns.replace_all(&org_contents, |caps: &Captures| {
        if let Some(title) = caps.get(1) {
            format!(
                "#show: project.with(title: \"{}\", authors: (\"{}\",))",
                title.as_str(),
                match caps.get(2) {
                    Some(author) => author.as_str(),
                    None => "Ziqin Gong",
                }
            )
        } else if let Some(headline) = caps.get(3) {
            format!("\n{}", "=".repeat(headline.len()))
        } else if let Some(_) = caps.get(4) {
            String::from("```")
        } else if let Some(_) = caps.get(5) {
            String::from("")
        } else if let Some(_) = caps.get(6) {
            String::from("")
        } else if let Some(italic) = caps.get(7) {
            format!("_{}_", italic.as_str())
        } else if let Some(orgcite) = caps.get(8) {
            format!("#underline[{}]", orgcite.as_str())
        } else if let Some(quote) = caps.get(9) {
            format!("\"{}\"", quote.as_str())
        } else {
            let citation = match caps.get(10) {
                Some(info) => String::from(info.as_str()),
                None => String::from("error"),
            };

            citation
        }
    });

    let mut output = String::from(
        "#let project(title: \"\", authors: (), date: none, body) = {
  // Set the document's basic properties.
  set document(author: authors, title: title)
  set page(numbering: \"1\", number-align: center)

  // Save heading and body font families in variables.
  let body-font = \"New Computer Modern\"
  let sans-font = \"New Computer Modern Sans\"

  // Set body font family.
  set text(font: body-font, lang: \"en\")
  show math.equation: set text(weight: 400)
  show heading: set text(font: sans-font)

  // Title row.
  align(center)[
    #block(text(font: sans-font, weight: 700, 1.75em, title))
    #v(1em, weak: true)
    #date
  ]

  // Author information.
  pad(
    top: 0.5em,
    bottom: 0.5em,
    x: 2em,
    grid(
      columns: (1fr,) * calc.min(3, authors.len()),
      gutter: 1em,
      ..authors.map(author => align(center, strong(author))),
    ),
  )

  // Main body.
  set par(justify: true)

  body
}
",
    );
    output.push_str(&result);
    output.push_str("\n\n#bibliography(\"refs.bib\")");

    if let Some(typst_path) = config.typst_file {
        let mut out_file = match File::create(typst_path) {
            Ok(file) => file,
            Err(_) => return Err("Failed to open output typst file"),
        };
        out_file
            .write_all(output.as_bytes())
            .expect("Failed to write to typst file");
    } else {
        println!("{}", output);
    }

    Ok(())
}

pub fn print_config(config: &Config) {
    println!("Org file path: {}", config.org_file.display());
    if let Some(typst_path) = &config.typst_file {
        println!("Output path: {}", typst_path.display());
    } else {
        println!("Output path: stdout");
    }
}
