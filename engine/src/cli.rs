use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "assay",
    version,
    about = "Plan and execute repository checks with versioned evidence",
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(long, global = true, default_value = ".", value_name = "PATH")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run {
        #[arg(long, conflicts_with_all = ["check", "stage"])]
        base: Option<String>,
        #[arg(long)]
        stage: Option<String>,
        #[arg(long)]
        check: Option<String>,
        #[arg(long, conflicts_with_all = ["check", "stage"])]
        change: Option<String>,
        #[arg(long)]
        certify: bool,
    },
    Plan {
        #[arg(long, value_name = "REV")]
        base: String,
        #[arg(long)]
        change: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Map,
    Check,
    /// 只读就绪度：仓库根 / 清单 / 棘轮 / 证据目录 / 地图新鲜度 /
    /// **声明的命令能否解析到**（只看 PATH，不执行）
    Doctor {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// 只读导览：由清单 + 证据 + 棘轮算出**有序动作表**
    Next {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Version,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::{Cli, Command};

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_root_before_and_after_subcommand() {
        let before = Cli::try_parse_from(["assay", "--root", "repo", "check"]).unwrap();
        let after = Cli::try_parse_from(["assay", "check", "--root", "repo"]).unwrap();

        assert_eq!(before.root, after.root);
        assert!(matches!(before.command, Command::Check));
        assert!(matches!(after.command, Command::Check));
    }

    #[test]
    fn rejects_unknown_arguments_and_conflicting_run_filters() {
        assert!(Cli::try_parse_from(["assay", "check", "--unknown"]).is_err());
        assert!(Cli::try_parse_from(["assay", "run", "--base", "HEAD", "--stage", "pr"]).is_err());
    }
}
