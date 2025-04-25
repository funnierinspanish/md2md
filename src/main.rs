use clap::Parser;
use minijinja::{Environment, context};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "md2md")]
#[command(version = "0.0.1")]
#[command(about = "Converts Markdown to Markdown but using templates in between", long_about = None)]
struct Cli {
    /// The source file to be _templated_
    #[arg()]
    input_path: String,

    /// The directory containing the templates. Default: `templates`
    #[arg(short = 't', long = "templates-path", required = true)]
    templates: String,

    #[arg(short = 'o', long = "output-path", default_value = "out")]
    output: String,
}

fn main() {
    let cli = Cli::parse();

    let source_file_path = Path::new(&cli.input_path).to_path_buf();
    let templates_path = Path::new(&cli.templates).to_path_buf();
    let output_path = Path::new(&cli.output).to_path_buf();

    if !source_file_path.exists() {
        println!("Source file does not exist: {:?}", source_file_path);
        std::process::exit(1);
    }

    if source_file_path.is_dir() && !output_path.is_dir() {
        println!(
            "Source path is a directory: {:?}, but output path seems to be a file {:?}",
            source_file_path, output_path
        );
        std::process::exit(1);
    }

    if !templates_path.exists() {
        println!("Templates path does not exist: {:?}", templates_path);
        std::process::exit(1);
    }
    if output_path.is_dir() && !output_path.exists() {
        println!(
            "Output directory does not exist, creating: {:?}",
            output_path
        );
        fs::create_dir_all(&output_path)
            .unwrap_or_else(|e| panic!("Failed to create output directory: {}", e));
    }

    if !output_path.exists() {
        fs::create_dir_all(&output_path)
            .unwrap_or_else(|e| panic!("Failed to create output directory: {}", e));
    }

    let _ = templating_stuff(&source_file_path, &templates_path, &output_path);
}

fn templating_stuff(
    source_file_path: &Path,
    template_path: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    let mut source_file_names: Vec<String> = vec![];

    if source_file_path.is_dir() {
        for entry in fs::read_dir(source_file_path).expect("Failed to read source directory") {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = entry.path();
                let filename_os = entry.file_name();
                let name_str = filename_os
                    .to_str()
                    .expect(&format!("Invalid UTF-8 in filename {:?}", filename_os));
                source_file_names.push(String::from(name_str));
                let contents = fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
                env.add_template_owned(String::from(name_str), contents)
                    .unwrap_or_else(|e| panic!("Failed to add template `{}`: {}", name_str, e));
            }
        }
    } else if source_file_path.is_file() {
        let filename_os = source_file_path
            .file_name()
            .expect("Failed to get file name");
        let name_str = filename_os
            .to_str()
            .expect(&format!("Invalid UTF-8 in filename {:?}", filename_os));
        source_file_names.push(String::from(name_str));
        let contents = fs::read_to_string(source_file_path)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", source_file_path, e));
        env.add_template_owned(name_str.to_string(), contents)
            .unwrap_or_else(|e| panic!("Failed to add template `{}`: {}", name_str, e));
    }
    
    if let Ok(entries) = template_path.read_dir() {
        for entry in entries.flatten() {
            // Skip non-files
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.path();

            let filename_os = entry.file_name();
            let name_str = filename_os
                .to_str()
                .expect(&format!("Invalid UTF-8 in filename {:?}", filename_os));
            let template_file_path = format!("{}/{}", template_path.display(), name_str);
            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
            env.add_template_owned(template_file_path, contents)
                .unwrap_or_else(|e| panic!("Failed to add template `{}`: {}", name_str, e));
        }
    } else {
        let contents = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", template_path, e));
        env.add_template_owned(template_path.to_str().unwrap(), contents)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to add template path `{}`: {}",
                    template_path.display(),
                    e
                )
            });
    }

    if source_file_names.len() == 1 {
        let source_file_name = &source_file_names[0];
        let rendered_content = env
            .get_template(source_file_name)
            .expect("Failed to get template")
            .render(context! {})
            .expect("Failed to render template");
        write_to_file(output_path, source_file_name, rendered_content)?;
    } else if source_file_names.len() > 1 {
        for source_file_name in source_file_names {
            let rendered_content = env
                .get_template(&source_file_name)
                .expect("Failed to get template")
                .render(context! {})
                .expect("Failed to render template");
            write_to_file(output_path, &source_file_name, rendered_content)?;
        }
    } else {
        println!("No source files found");
        std::process::exit(1);
    }

    Ok(())
}

fn write_to_file(
    output_path: &Path,
    template_name: &str,
    rendered_content: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = match output_path.is_dir() {
        true => output_path.join(template_name),
        false => output_path.to_path_buf(),
    };

    fs::write(&out_path, rendered_content.as_bytes()).expect("Failed to write output file");

    println!("Output written to {:?}", out_path.display());
    Ok(())
}
