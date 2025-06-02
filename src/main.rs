use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use hyprland::data;
use hyprland::data::{Monitor, Monitors, Workspaces};
use hyprland::dispatch::DispatchType::{RenameWorkspace, Workspace};
use hyprland::dispatch::{Dispatch, WorkspaceIdentifierWithSpecial};
use hyprland::prelude::{HyprData, HyprDataActive};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Monitor workspaces of which will be controlled. By default, the active will be used
    monitor: Option<String>,

    /// Prefix for this dynamic workspace group. Will be used to name the workspaces
    #[arg(short, long, default_value = ":")]
    prefix: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
enum Position {
    Start,
    End,
    Next,
    Previous,
}

#[derive(Subcommand)]
enum Commands {
    /// create new workspace
    New {
        /// where the new workspace should be created
        position: Position,
    },
    /// find address of a workspace, which can be provided to a hyprctl dispatcher. May be an id or a name or whatever
    Find {
        /// where the new workspace should be created
        position: Position,
    },
}

struct Executor<'a> {
    cli: &'a Cli,
    monitor: &'a Monitor,
}

impl<'a> Executor<'a> {
    fn make_workspace_name(&self, pos: usize) -> String {
        format!("{}{:05}", self.cli.prefix, pos)
    }

    fn get_workspace_pos(&self, name: &str) -> Option<usize> {
        name.strip_prefix(&self.cli.prefix)
            .and_then(|m| m.parse().ok())
    }

    fn is_dyn_workspace(&self, name: &str) -> bool {
        name.starts_with(&self.cli.prefix)
    }

    fn execute(&self) -> anyhow::Result<()> {
        match self.cli.command {
            Commands::New { position } => {
                let workspaces = self.find_dyn_workspaces()?;
                let pos = self.resolve_pos(position, &workspaces);
                self.insert_new_workspace(&workspaces, pos)?;
            }
            Commands::Find { position } => {
                self.find_workspace(position)?;
            }
        }

        Ok(())
    }

    fn resolve_pos(&self, position: Position, workspaces: &Vec<data::Workspace>) -> usize {
        match position {
            Position::Start => 0,
            Position::End => workspaces.len(),
            Position::Next => {
                if let Some(pos) = self.get_workspace_pos(&self.monitor.active_workspace.name) {
                    pos + 1
                } else {
                    workspaces.len()
                }
            }

            Position::Previous => {
                if let Some(pos) = self.get_workspace_pos(&self.monitor.active_workspace.name) {
                    pos
                } else {
                    0
                }
            }
        }
    }

    fn new_workspace(&self, pos: usize) -> anyhow::Result<()> {
        let new_name = self.make_workspace_name(pos);
        Dispatch::call(Workspace(WorkspaceIdentifierWithSpecial::Name(&new_name)))?;
        Ok(())
    }

    fn find_dyn_workspaces(&self) -> anyhow::Result<Vec<data::Workspace>> {
        let mut workspaces = Workspaces::get()?
            .into_iter()
            .filter(|w| w.monitor_id == self.monitor.id && self.is_dyn_workspace(&w.name))
            .collect::<Vec<_>>();
        workspaces.sort_unstable_by(|l, r| l.name.cmp(&r.name));
        Ok(workspaces)
    }

    fn insert_new_workspace(
        &self,
        workspaces: &Vec<data::Workspace>,
        pos: usize,
    ) -> anyhow::Result<()> {
        for (i, w) in workspaces.iter().enumerate() {
            let p = if i < pos { i } else { i + 1 };
            Dispatch::call(RenameWorkspace(w.id, Some(&self.make_workspace_name(p))))?;
        }

        self.new_workspace(pos)?;

        Ok(())
    }

    fn find_workspace(&self, position: Position) -> anyhow::Result<()> {
        let mut workspaces = Workspaces::get()?
            .into_iter()
            .filter(|w| w.monitor_id == self.monitor.id)
            .collect::<Vec<_>>();
        workspaces.sort_unstable_by(|l, r| l.name.cmp(&r.name));

        let idx = match position {
            Position::Start => 0,
            Position::End => workspaces.len() - 1,
            Position::Next => {
                let idx = workspaces
                    .iter()
                    .position(|w| w.id == self.monitor.active_workspace.id)
                    .unwrap();
                (idx + 1) % workspaces.len()
            }
            Position::Previous => {
                let idx = workspaces
                    .iter()
                    .position(|w| w.id == self.monitor.active_workspace.id)
                    .unwrap();
                (workspaces.len() + idx - 1) % workspaces.len()
            }
        };

        println!("name:{}", workspaces[idx].name);

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(monitor_name) = &cli.monitor {
        for monitor in Monitors::get()? {
            if &monitor.name == monitor_name {
                Executor {
                    cli: &cli,
                    monitor: &monitor,
                }
                .execute()?;
                return Ok(());
            }
        }

        Err(anyhow!("monitor not found"))
    } else {
        Executor {
            cli: &cli,
            monitor: &Monitor::get_active()?,
        }
        .execute()?;
        Ok(())
    }
}
