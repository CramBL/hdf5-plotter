use {
    clap::{
        builder::styling::{AnsiColor, Effects, Styles},
        command, ArgAction, Args, Parser, Subcommand, ValueEnum,
    },
    std::{fmt, path::PathBuf},
    stderrlog::LogLevelNum,
    strum_macros::{Display, EnumIter},
};

pub mod misc;

static BIN_NAME: &str = "ploth5";

#[derive(Debug, Parser)]
#[command(name = "HDF5 Plotter", version, styles = misc::cli_styles())]
#[command(bin_name = BIN_NAME)]
pub struct Config {
    /// Accepted subcommands, e.g. `listen`
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// Pass many times for more log output
    ///
    /// By default, it'll report errors, warnings and info,
    /// `-v` enables debug messages, `-vv` for trace messages.
    #[arg(short, long, action = ArgAction::Count, default_value_t = 0, global = true)]
    pub verbose: u8,

    /// Silence all log output, this will lead to better performance.
    #[arg(short, long, action = ArgAction::SetTrue, conflicts_with("verbose"), global = true, env = "QFT_QUIET")]
    pub quiet: bool,

    #[arg(
        long,
        require_equals = true,
        value_name = "WHEN",
        default_value_t = clap::ColorChoice::Auto,
        default_missing_value = "always",
        value_enum,
        global = true
    )]
    pub color: clap::ColorChoice,

    /// Generate completion scripts for the specified shell.
    /// Note: The completion script is printed to stdout
    #[arg(
        long = "completions",
        value_hint = clap::ValueHint::Other,
        value_name = "SHELL"
    )]
    pub completions: Option<clap_complete::Shell>,
}

impl Config {
    pub fn init() -> anyhow::Result<Self> {
        let cfg = Self::parse();

        let log_level: LogLevelNum = match cfg.verbose {
            0 => LogLevelNum::Info,
            1 => LogLevelNum::Debug,
            255 => LogLevelNum::Off,
            _ => LogLevelNum::Trace,
        };

        let log_color_when: stderrlog::ColorChoice = match cfg.color {
            clap::ColorChoice::Auto => stderrlog::ColorChoice::Auto,
            clap::ColorChoice::Always => stderrlog::ColorChoice::Always,
            clap::ColorChoice::Never => stderrlog::ColorChoice::Never,
        };

        stderrlog::new()
            .verbosity(log_level)
            .quiet(cfg.quiet)
            .color(log_color_when)
            .init()?;

        Ok(cfg)
    }

    /// Generate completion scripts for the specified shell.
    pub fn generate_completion_script(shell: clap_complete::Shell) {
        use clap::CommandFactory;
        clap_complete::generate(
            shell,
            &mut Config::command(),
            BIN_NAME,
            &mut std::io::stdout(),
        );
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Plot(PlotArgs),
    Inspect(InspectArgs),
    TestSwMr,
}

#[derive(Debug, Args, Clone)]
#[command(flatten_help = true)]
pub struct PlotArgs {
    /// Hdf5 file
    #[arg(required(true))]
    pub src_hdf5: PathBuf,

    #[arg(short, long)]
    pub dataset_name: Option<String>,
}

#[derive(Debug, Args, Clone)]
#[command(flatten_help = true)]
pub struct InspectArgs {
    #[arg(required(true))]
    pub src_hdf5: PathBuf,
}
