use clap::{Parser, ValueEnum};
use prism_rs::parser::parse_dtmc;

#[derive(ValueEnum, Clone, Debug)]
enum ModelType {
    DTMC,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    model_type: ModelType,

    #[arg(long)]
    model: String,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    println!("== Prism-rs ==");

    match args.model_type {
        ModelType::DTMC => {
            println!("Parsing DTMC model from file: {}", args.model);
            let model_str =
                std::fs::read_to_string(&args.model).expect("Failed to read model file");
            match parse_dtmc(&model_str) {
                Ok(model) => {
                    println!("Successfully parsed DTMC model:");
                    println!("{:#?}", model);
                }
                Err(e) => {
                    eprintln!("Failed to parse DTMC model: {}", e);
                }
            }
        }
    }
}
