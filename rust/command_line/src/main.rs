use calamine::{Reader, Xlsx};
use clap::Parser;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use symbolic_regression::expression_tree::ExpressionTree;
use symbolic_regression::model::default::{
    GenerationSize, Individual, Model, ModelResult, StopCriterion, StopReason,
};
use symbolic_regression::model::settings::Settings;
use symbolic_regression::model::{FitnessError, InputData};

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
    log_path: Option<PathBuf>,
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

struct RunResult {
    model_result: Result<ModelResult, FitnessError>,
    generations: Vec<Vec<Rc<Individual>>>,
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
    let output_variable = input_data.variables[input_data.variables.len() - 1].to_owned();
    let RunResult {
        model_result,
        generations,
    } = run_model(
        &cli,
        settings,
        input_data,
        stop_criterion,
        generation_size,
        auxiliary_expression_trees,
    );
    print_model_result(output_variable, model_result);
    if let Some(path) = &cli.log_path {
        output_log(path, generations);
    }
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

fn run_model(
    cli: &Cli,
    settings: Settings,
    input_data: InputData,
    stop_criterion: StopCriterion,
    generation_size: GenerationSize,
    auxiliary_expression_trees: Vec<ExpressionTree>,
) -> RunResult {
    let log = !cli.log_path.is_none();
    let log_every = cli.log_every;
    let mut generation_counter = 0;
    let generations = Rc::new(RefCell::new(vec![]));
    let generation_copy = Rc::clone(&generations);
    let mut model = Model::new(
        settings,
        input_data,
        stop_criterion,
        generation_size,
        auxiliary_expression_trees,
        Some(Box::new(move |generation| {
            if log && generation_counter % log_every == 0 {
                generation_copy
                    .borrow_mut()
                    .push(generation.iter().cloned().collect::<Vec<Rc<Individual>>>())
            }
            generation_counter += 1;
        })),
    );
    let model_result = model.run();
    drop(model);
    RunResult {
        model_result,
        generations: generations.take(),
    }
}

fn print_model_result(output_variable: String, model_result: Result<ModelResult, FitnessError>) {
    match model_result {
        Ok(result) => {
            println!(
                "Result function: {} = {}",
                output_variable, result.individual.expression_tree
            );
            match &result.stop_reason {
                StopReason::Error(error) => println!("The reason for the stop is an error equal to {}", error),
                StopReason::WithoutImprovement(without_improvement) => println!(
                    "The reason for the stop is {} generations without improvements with an error equal to {}",
                    without_improvement.generation_number,
                    without_improvement.error
                ),
                StopReason::GenerationNumber(generation_number) => println!(
                    "The reason for stopping is the maximum number of generations, equal to {}",
                    generation_number
                )
            };
        }
        Err(err) => exit_with_error(&format!("{}", err)),
    }
}

fn output_log(log_path: &PathBuf, generations: Vec<Vec<Rc<Individual>>>) {
    let file = match File::create(log_path) {
        Ok(file) => file,
        Err(err) => exit_with_error(&format!(r#"Can't create log file: "{}"."#, err)),
    };
    if let Err(err) = serde_json::to_writer_pretty(file, &generations) {
        exit_with_error(&format!(r#"Can't serialize log generations: "{}"."#, err))
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1)
}
