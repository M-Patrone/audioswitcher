use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct AudioSwitcherArgs {
    #[clap(subcommand)]
    pub entity_type: EntityType,

    /// enable extensive logging
    #[clap(long, global = true)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    /// Shows all audio devices (including speaker and microphone)
    Show,

    /// Set with dropdown (useful if you don't know the device id)
    Select(SelectCommand),

    /// Get the active speaker and microphone on windows
    Active,

    /// Set the device which you want to display
    Set(SetCommand),
}

#[derive(Debug, Args)]
pub struct SetCommand {
    /// speaker device id which should be set
    pub speaker_device_id: String,

    /// microphone device id which should be set. If it is not provided, the speaker device id will be set in ms teams
    pub microphone_device_id: Option<String>,

    /// Option to disable changing the Windows audio device (default: true)
    #[clap(long, action = clap::ArgAction::SetFalse)]
    pub change_windows_audio_device: bool,
}

#[derive(Debug, Args)]
pub struct SelectCommand {
    /// Option to disable changing the Windows audio device (default: true)
    #[clap(long, action = clap::ArgAction::SetFalse)]
    pub change_windows_audio_device: bool,
}
