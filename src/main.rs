use anyhow::{anyhow, bail};
use clap::{builder::EnumValueParser, Parser, Subcommand, ValueEnum};
use console::style;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use reqwest::Url;
use std::fs::File;
use std::io::copy;
use std::{
    env,
    fs::{self},
    path::PathBuf,
    str::FromStr,
};
use tokio::runtime::Runtime;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[arg(default_value = "apepkuss")]
    name: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Start {
        #[arg(
            short = 'm',
            long = "model",
            help = "Url to the gguf model",
            ignore_case = true
        )]
        model: Option<String>,
        #[arg(
            short = 'p',
            long = "prompt-template",
            help = "Type of prompt template for the gguf model",
            requires = "model",
            value_parser = EnumValueParser::<PromptTemplateType>::new(),
        )]
        prompt_template: Option<PromptTemplateType>,
        #[arg(
            short = 'r',
            long = "reverse-prompt",
            help = "Halt generation at PROMPT, return control",
            requires = "model"
        )]
        reverse_prompt: Option<String>,
        #[arg(
            short = 'c',
            long = "context-size",
            help = "Prompt context size",
            requires = "model"
        )]
        context_size: Option<u64>,
    },
    Stop,
}

const PROMPT_TEMPLATES: [&str; 20] = [
    "llama-2-chat",
    "mistral-instruct",
    "mistrallite",
    "openchat",
    "codellama-instruct",
    "human-asistant",
    "vicuna-1.0-chat",
    "vicuna-1.1-chat",
    "vicuna-llava",
    "chatml",
    "baichuan-2",
    "wizard-coder",
    "zephyr",
    "stablelm-zephyr",
    "intel-neural",
    "deepseek-chat",
    "deepseek-coder",
    "solar-instruct",
    "phi-2-chat",
    "phi-2-instruct",
];

#[derive(Clone, Debug, Copy, PartialEq, Eq, ValueEnum)]
pub enum PromptTemplateType {
    Llama2Chat,
    MistralInstruct,
    MistralLite,
    OpenChat,
    CodeLlama,
    CodeLlamaSuper,
    HumanAssistant,
    VicunaChat,
    Vicuna11Chat,
    VicunaLlava,
    ChatML,
    Baichuan2,
    WizardCoder,
    Zephyr,
    StableLMZephyr,
    IntelNeural,
    DeepseekChat,
    DeepseekCoder,
    SolarInstruct,
    Phi2Chat,
    Phi2Instruct,
    GemmaInstruct,
}
impl FromStr for PromptTemplateType {
    type Err = anyhow::Error;

    fn from_str(template: &str) -> std::result::Result<Self, Self::Err> {
        match template {
            "llama-2-chat" => Ok(PromptTemplateType::Llama2Chat),
            "mistral-instruct" => Ok(PromptTemplateType::MistralInstruct),
            "mistrallite" => Ok(PromptTemplateType::MistralLite),
            "codellama-instruct" => Ok(PromptTemplateType::CodeLlama),
            "codellama-super-instruct" => Ok(PromptTemplateType::CodeLlamaSuper),
            "belle-llama-2-chat" => Ok(PromptTemplateType::HumanAssistant),
            "human-assistant" => Ok(PromptTemplateType::HumanAssistant),
            "vicuna-1.0-chat" => Ok(PromptTemplateType::VicunaChat),
            "vicuna-1.1-chat" => Ok(PromptTemplateType::Vicuna11Chat),
            "vicuna-llava" => Ok(PromptTemplateType::VicunaLlava),
            "chatml" => Ok(PromptTemplateType::ChatML),
            "openchat" => Ok(PromptTemplateType::OpenChat),
            "baichuan-2" => Ok(PromptTemplateType::Baichuan2),
            "wizard-coder" => Ok(PromptTemplateType::WizardCoder),
            "zephyr" => Ok(PromptTemplateType::Zephyr),
            "stablelm-zephyr" => Ok(PromptTemplateType::StableLMZephyr),
            "intel-neural" => Ok(PromptTemplateType::IntelNeural),
            "deepseek-chat" => Ok(PromptTemplateType::DeepseekChat),
            "deepseek-coder" => Ok(PromptTemplateType::DeepseekCoder),
            "solar-instruct" => Ok(PromptTemplateType::SolarInstruct),
            "phi-2-chat" => Ok(PromptTemplateType::Phi2Chat),
            "phi-2-instruct" => Ok(PromptTemplateType::Phi2Instruct),
            "gemma-instruct" => Ok(PromptTemplateType::GemmaInstruct),
            _ => bail!(template.to_string()),
        }
    }
}
impl std::fmt::Display for PromptTemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptTemplateType::Llama2Chat => write!(f, "llama-2-chat"),
            PromptTemplateType::MistralInstruct => write!(f, "mistral-instruct"),
            PromptTemplateType::MistralLite => write!(f, "mistrallite"),
            PromptTemplateType::OpenChat => write!(f, "openchat"),
            PromptTemplateType::CodeLlama => write!(f, "codellama-instruct"),
            PromptTemplateType::HumanAssistant => write!(f, "human-asistant"),
            PromptTemplateType::VicunaChat => write!(f, "vicuna-1.0-chat"),
            PromptTemplateType::Vicuna11Chat => write!(f, "vicuna-1.1-chat"),
            PromptTemplateType::VicunaLlava => write!(f, "vicuna-llava"),
            PromptTemplateType::ChatML => write!(f, "chatml"),
            PromptTemplateType::Baichuan2 => write!(f, "baichuan-2"),
            PromptTemplateType::WizardCoder => write!(f, "wizard-coder"),
            PromptTemplateType::Zephyr => write!(f, "zephyr"),
            PromptTemplateType::StableLMZephyr => write!(f, "stablelm-zephyr"),
            PromptTemplateType::IntelNeural => write!(f, "intel-neural"),
            PromptTemplateType::DeepseekChat => write!(f, "deepseek-chat"),
            PromptTemplateType::DeepseekCoder => write!(f, "deepseek-coder"),
            PromptTemplateType::SolarInstruct => write!(f, "solar-instruct"),
            PromptTemplateType::Phi2Chat => write!(f, "phi-2-chat"),
            PromptTemplateType::Phi2Instruct => write!(f, "phi-2-instruct"),
            PromptTemplateType::CodeLlamaSuper => write!(f, "codellama-super-instruct"),
            PromptTemplateType::GemmaInstruct => write!(f, "gemma-instruct"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            model,
            prompt_template,
            reverse_prompt,
            context_size,
        } => {
            // gguf model
            command_start(model, prompt_template, reverse_prompt, context_size);

            // start Qdrant

            // start api-server
        }
        Commands::Stop => {
            // stop api-server

            // stop Qdrant

            unimplemented!("Stop command not implemented")
        }
    }

    Ok(())
}

fn command_start(
    model: Option<String>,
    prompt_template: Option<PromptTemplateType>,
    reverse_prompt: Option<String>,
    context_size: Option<u64>,
) -> anyhow::Result<()> {
    let gguf_model = match model {
        Some(model) => {
            println!("Model: {}", model);
            "fake.gguf".to_string()
        }
        None => {
            // check cached models
            let cwd = env::current_dir().unwrap();
            let entries = fs::read_dir(cwd).unwrap();
            let mut cached_models = entries
                .filter_map(|res| {
                    res.ok().and_then(|e| {
                        e.path()
                            .file_name()
                            .and_then(|n| n.to_str().map(|s| String::from(s)))
                            .filter(|s| s.ends_with(".gguf"))
                    })
                })
                .collect::<Vec<String>>();

            let mut selected = String::new();
            if !cached_models.is_empty() {
                cached_models.push("Or choose one from: https://huggingface.co/second-state?sort_models=modified#models or https://huggingface.co/models?sort=trending&search=gguf".to_string());
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a chached model")
                    .default(0)
                    .items(&cached_models[..])
                    .interact_opt()?;

                selected = match selection {
                    Some(idx) => cached_models[idx].clone(),
                    _ => panic!("Fatal: No selection!"),
                };
            }

            if selected.ends_with(".gguf") {
                selected
            } else {
                // provide a model url to download
                let model_url = dialoguer::Input::<String>::new()
                    .with_prompt("Enter the model url")
                    .interact()?;

                // download the model from the url
                download_model(model_url)?
            }
        }
    };

    let prompt_template: PromptTemplateType = match prompt_template {
        Some(prompt_template) => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a prompt template")
                .default(0)
                .items(&PROMPT_TEMPLATES[..])
                .interact_opt()?;

            match selection {
                Some(idx) => {
                    let x = PROMPT_TEMPLATES[idx];
                    <PromptTemplateType as FromStr>::from_str(x)?
                }
                _ => panic!("Fatal: No selection!"),
            }
        }
        None => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a prompt template")
                .default(0)
                .items(&PROMPT_TEMPLATES[..])
                .interact_opt()?;

            match selection {
                Some(idx) => {
                    let x = PROMPT_TEMPLATES[idx];
                    <PromptTemplateType as FromStr>::from_str(x)?
                }
                _ => panic!("Fatal: No selection!"),
            }
        }
    };

    // let directory = matches.value_of("directory").unwrap_or(".");

    let cwd = env::current_dir().unwrap();

    let entries = fs::read_dir(cwd).unwrap();

    let mut files = entries
        .filter_map(|res| {
            res.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
                    .filter(|s| s.ends_with(".gguf"))
            })
        })
        .collect::<Vec<String>>();

    // if files.is_empty() {
    //     println!("No *.gguf files found in the current directory.");
    //     return;
    // }

    // files.sort();

    // let selection = Select::new()
    //     .items(&files)
    //     .default(0)
    //     .interact_on_opt(&Term::stdout())
    //     .unwrap();

    // match selection {
    //     Some(index) => println!("You selected: {}", files[index]),
    //     None => println!("No file selected."),
    // }

    Ok(())
}

// Download the model from the given url
fn download_model(url: String) -> anyhow::Result<String> {
    let url = Url::parse(&url)?;
    let response = reqwest::blocking::get(url)?;

    // let mut filename = String::new();
    let (mut dest, fname) = {
        let fname = response
            .url()
            .path_segments()
            .and_then(std::iter::Iterator::last)
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .ok_or(anyhow!("No filename found in the url to download"))?;
        (File::create(fname)?, fname.to_string())
    };

    let content = response.bytes()?;
    copy(&mut content.as_ref(), &mut dest)?;

    Ok(fname)
}
