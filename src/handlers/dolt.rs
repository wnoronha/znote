use anyhow::Result;
use std::path::Path;

use crate::commands::DoltCommands;
use crate::storage::dolt::DoltStorage;

pub fn run(data_dir: &Path, command: &DoltCommands) -> Result<()> {
    match command {
        DoltCommands::Sync => {
            let db = DoltStorage::new(data_dir);
            db.init_db()?;
            db.import_from_fs()?;
            println!("Synchronized markdown data to Dolt backend.");
        }
        DoltCommands::RemoteAdd { name, url } => {
            let db = DoltStorage::new(data_dir);
            db.add_remote(name, url)?;
            println!("Added remote {} at {}", name, url);
        }
        DoltCommands::Pull { remote } => {
            let db = DoltStorage::new(data_dir);
            db.pull(remote)?;
            println!("Pulled from {}", remote);
        }
        DoltCommands::Push { remote } => {
            let db = DoltStorage::new(data_dir);
            db.push(remote)?;
            println!("Pushed to {}", remote);
        }
    }
    Ok(())
}
