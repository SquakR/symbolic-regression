use calamine::{Reader, Xlsx};
use clap::Parser;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use symbolic_regression::expression_tree::ExpressionTree;
use symbolic_regression::model::settings::Settings;
use symbolic_regression::model::{GenerationSize, InputData, StopCriterion};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to json or xlsx file with input data.
    #[clap(long, short, value_parser)]
    input_data_path: PathBuf,
    /// Path to json configuration file.
    #[clap(long, short, value_parser)]
    config_path: PathBuf,
    /// Path to json file for logging.
    #[clap(long, short, value_parser)]
    log_path: Option<String>,
    /// Log every <LOG> generation.
    #[clap(long, short = 'e', value_parser, default_value = "25")]
    log_every: usize,
}

#[derive(Deserialize)]
struct Config {
    stop_criterion: StopCriterion,
    generation_size: GenerationSize,
    auxiliary_expressions: Vec<String>,
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1)
}

fn read_input_data(cli: &Cli) -> InputData {
    match cli.input_data_path.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "json" => match fs::read_to_string(&cli.input_data_path) {
                Ok(json) => match InputData::from_json(&json) {
                    Ok(input_data) => input_data,
                    Err(err) => exit_with_error(&format!(
                        r#"Can't crete new InputData from json string: "{}"."#,
                        err
                    )),
                },
                Err(err) => exit_with_error(&format!(r#"Can't read input data file: "{}"."#, err)),
            },
            "xlsx" => match calamine::open_workbook::<Xlsx<_>, _>(&cli.input_data_path) {
                Ok(mut workbook) => match InputData::from_worksheet_range(
                    workbook
                        .worksheet_range(&workbook.sheet_names()[0].to_owned())
                        .unwrap()
                        .unwrap(),
                ) {
                    Ok(input_data) => input_data,
                    Err(err) => exit_with_error(&format!(
                        r#"Can't crete new InputData from first sheet: "{}"."#,
                        err
                    )),
                },
                Err(err) => exit_with_error(&format!(r#"Can't read input data file: "{}"."#, err)),
            },
            _ => exit_with_error(&format!(
                r#"Invalid input data file extension, expected "json" or "xlsx", but received "{}"."#,
                extension.to_str().unwrap()
            )),
        },
        None => exit_with_error(
            r#"Invalid input data file extension, expected "json" or "xlsx", but received None."#,
        ),
    }
}

fn read_config(cli: &Cli) -> Config {
    let file = match File::open(&cli.config_path) {
        Ok(file) => file,
        Err(err) => exit_with_error(&format!(r#"Can't read configuration file: "{}"."#, err)),
    };
    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(config) => config,
        Err(err) => exit_with_error(&format!(r#"Can't parse configuration file: "{}"."#, err)),
    }
}

fn parse_expression_trees(
    settings: &Settings,
    expressions: Vec<String>,
    variables: &[String],
) -> Vec<ExpressionTree> {
    let variables_set = HashSet::<&str>::from_iter(variables.iter().map(|v| v.as_str()));
    let mut expression_trees = vec![];
    for expression in expressions {
        match ExpressionTree::parse(&expression, settings) {
            Ok(mut expression_tree) => {
                let expression_tree_variables = HashSet::<&str>::from_iter(
                    expression_tree.variables.iter().map(|v| v.as_str()),
                );
                if expression_tree_variables.is_subset(&variables_set) {
                    expression_tree.variables = variables.iter().cloned().collect::<Vec<String>>();
                    expression_trees.push(expression_tree);
                } else {
                    exit_with_error(&format!(
                        r#"The expression tree for "{}" expression contains {:?} variables, but the input data contains "{:?}" ones."#,
                        expression, expression_tree.variables, variables
                    ));
                }
            }
            Err(err) => exit_with_error(&format!(
                r#"Can't parse expression "{}": "{}"."#,
                expression, err
            )),
        }
    }
    expression_trees
}

fn main() {
    let cli = Cli::parse();
    let settings = Settings::default();
    let input_data = read_input_data(&cli);
    let Config {
        stop_criterion,
        generation_size,
        auxiliary_expressions,
    } = read_config(&cli);
    let auxiliary_expression_trees = parse_expression_trees(
        &settings,
        auxiliary_expressions,
        &input_data.variables[0..input_data.variables.len() - 1],
    );
    println!("{:?}", input_data);
    println!("{:?}", stop_criterion);
    println!("{:?}", generation_size);
    println!("{:?}", auxiliary_expression_trees);
}
