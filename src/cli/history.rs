//! History command handler
//!
//! View and manage generation history.

use crate::error::Result;
use crate::history::History;
use clap::{Args, Subcommand};

/// History command arguments
#[derive(Args)]
pub struct HistoryArgs {
    #[command(subcommand)]
    pub command: Option<HistoryCommand>,

    /// Number of entries to show (default: 10)
    #[arg(short = 'n', long, default_value = "10")]
    pub count: usize,
}

/// History subcommands
#[derive(Subcommand)]
pub enum HistoryCommand {
    /// List history entries
    List {
        /// Number of entries to show
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,
    },
    /// Show a specific entry
    Show {
        /// Entry ID
        id: String,
    },
    /// Delete a history entry
    Delete {
        /// Entry ID
        id: String,
    },
    /// Clear all history
    Clear,
    /// Show favorites only
    Favorites,
}

/// Run the history command
pub async fn run(args: HistoryArgs) -> Result<()> {
    let command = args.command.unwrap_or(HistoryCommand::List { count: args.count });

    match command {
        HistoryCommand::List { count } => list_history(count),
        HistoryCommand::Show { id } => show_entry(&id),
        HistoryCommand::Delete { id } => delete_entry(&id),
        HistoryCommand::Clear => clear_history(),
        HistoryCommand::Favorites => show_favorites(),
    }
}

/// List recent history entries
fn list_history(count: usize) -> Result<()> {
    let history = History::load()?;

    if history.is_empty() {
        println!("No history entries.");
        return Ok(());
    }

    println!("Recent generations ({} of {}):\n", count.min(history.len()), history.len());

    for entry in history.recent(count) {
        let name = entry.name.as_deref().unwrap_or("(unnamed)");
        let favorite = if entry.favorite { " *" } else { "" };
        let timestamp = &entry.response.metadata.timestamp;

        println!(
            "  {} - {}{}\n    Center: ({:.4}, {:.4}) | Radius: {}m\n    {}\n",
            &entry.response.id[..8],
            name,
            favorite,
            entry.response.request.lat,
            entry.response.request.lng,
            entry.response.request.radius,
            timestamp
        );
    }

    Ok(())
}

/// Show a specific history entry
fn show_entry(id: &str) -> Result<()> {
    let history = History::load()?;

    // Find entry by partial ID match
    let entry = history
        .entries()
        .iter()
        .find(|e| e.response.id.starts_with(id))
        .ok_or_else(|| crate::error::Error::Config(format!("Entry not found: {}", id)))?;

    let name = entry.name.as_deref().unwrap_or("(unnamed)");
    let favorite = if entry.favorite { " [favorite]" } else { "" };

    println!("Entry: {}{}", name, favorite);
    println!("ID: {}", entry.response.id);
    println!("Timestamp: {}", entry.response.metadata.timestamp);
    println!("\nRequest:");
    println!("  Center: ({}, {})", entry.response.request.lat, entry.response.request.lng);
    println!("  Radius: {}m", entry.response.request.radius);
    println!("  Mode: {:?}", entry.response.request.mode);
    println!("  Backend: {}", entry.response.request.backend);

    println!("\nResults:");
    for (anomaly_type, winner) in &entry.response.winners {
        let z_info = winner.result.z_score
            .map(|z| format!(" (z={:.2})", z))
            .unwrap_or_default();
        println!(
            "  {}: ({:.6}, {:.6}){}",
            anomaly_type,
            winner.result.coords.lat,
            winner.result.coords.lng,
            z_info
        );
    }

    if let Some(notes) = &entry.notes {
        println!("\nNotes: {}", notes);
    }

    Ok(())
}

/// Delete a history entry
fn delete_entry(id: &str) -> Result<()> {
    let mut history = History::load()?;

    // Find entry by partial ID match
    let full_id = history
        .entries()
        .iter()
        .find(|e| e.response.id.starts_with(id))
        .map(|e| e.response.id.clone())
        .ok_or_else(|| crate::error::Error::Config(format!("Entry not found: {}", id)))?;

    history.remove(&full_id);
    history.save()?;

    println!("Deleted entry: {}", full_id);
    Ok(())
}

/// Clear all history
fn clear_history() -> Result<()> {
    let mut history = History::load()?;
    let count = history.len();

    history.clear();
    history.save()?;

    println!("Cleared {} history entries.", count);
    Ok(())
}

/// Show favorite entries only
fn show_favorites() -> Result<()> {
    let history = History::load()?;
    let favorites = history.favorites();

    if favorites.is_empty() {
        println!("No favorite entries.");
        return Ok(());
    }

    println!("Favorite generations ({}):\n", favorites.len());

    for entry in favorites {
        let name = entry.name.as_deref().unwrap_or("(unnamed)");
        let timestamp = &entry.response.metadata.timestamp;

        println!(
            "  {} - {}\n    Center: ({:.4}, {:.4}) | Radius: {}m\n    {}\n",
            &entry.response.id[..8],
            name,
            entry.response.request.lat,
            entry.response.request.lng,
            entry.response.request.radius,
            timestamp
        );
    }

    Ok(())
}
