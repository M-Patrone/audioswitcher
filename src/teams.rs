use directories::BaseDirs;
use log::{error, info};
use serde_json::{Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::{Result, Write};
use std::os::windows::process;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

//ms teams settings are in the folder `%LocalAppData%\Packages\MSTeams_8wekyb3d8bbwe\LocalCache\Microsoft\MSTeams`
pub fn set_device_ms_teams(speaker_device_id: String, microphone_device_id: String) {
    if let Some(base_dirs) = BaseDirs::new() {
        let path = base_dirs.config_local_dir();
        let application_path = "Packages\\MSTeams_8wekyb3d8bbwe\\LocalCache\\Microsoft\\MSTeams";

        let full_path = path.join(application_path).join("cmd_settings.json");

        let content_unmodified = fs::read_to_string(full_path.clone())
            .expect("could not open the cmd_settings.json file");

        match modify_ms_teams_settings(speaker_device_id, microphone_device_id, content_unmodified)
        {
            Some(s) => {
                let p = Path::new(&full_path);
                let dir = p.parent().expect("Could not open the ms teams folder");
                let mut tmp = NamedTempFile::new_in(dir).unwrap();
                tmp.write_all(s.as_bytes()).unwrap();
                tmp.flush().unwrap();
                tmp.as_file().sync_all().unwrap();

                tmp.persist(p).unwrap();
                restart_ms_teams_client();
            }
            None => {
                println!(
                    "Could not write the settings to ms teams. Start the program with --verbose"
                );
            }
        }
    }
}

fn restart_ms_teams_client() {
    let _ = Command::new("taskkill")
        .args(["/IM", "ms-teams.exe", "/F"])
        .output()
        .expect("Error on closing teams client");
    let mut child = Command::new("ms-teams.exe").spawn().unwrap();
}

fn modify_ms_teams_settings(
    speaker_device_id: String,
    microphone_device_id: String,
    content: String,
) -> Option<String> {
    let unmodified_content = content.clone();
    let mut value: serde_json::Value =
        serde_json::from_str(&content).expect("Could not parse the cmd_settings.json as json file");

    if let Some(obj) = value.get_mut("calling_selected_speaker_device") {
        obj["Speaker"] = json!(speaker_device_id);
    }

    if let Some(obj) = value.get_mut("calling_selected_microphone_device") {
        obj["Microphone"] = json!(microphone_device_id);
    }

    if let Some(obj) = value.get_mut("calling_user_selected_devices") {
        obj["Speaker"] = json!(speaker_device_id);
        obj["Microphone"] = json!(microphone_device_id);
    }

    match write_backup_file(unmodified_content) {
        Ok(_) => {
            return Some(Value::to_string(&value));
        }
        Err(e) => {
            error!("could not write backup file {}", e);
        }
    }
    println!("value which has changed: {}", value);
    None
}

fn write_backup_file(content: String) -> Result<()> {
    let mut file = fs::File::create("cmd_settings_backup.json")?;
    file.write_all(content.as_bytes())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_modify_teams_settings() {
        let configuration = get_configuration();

        let modify_content = modify_ms_teams_settings(
            "speaker".to_string(),
            "microphone".to_string(),
            configuration,
        );
        assert_eq!(modify_content.is_some(), true);
        let res: Value = serde_json::from_str(&modify_content.unwrap()).unwrap();
        let exp_result: Value = serde_json::from_str(&get_result()).unwrap();

        assert_eq!(res, exp_result);
    }

    fn get_configuration() -> String {
        let ms_teams_conf = r#"{
  "calling_selected_camera_device": {
    "Camera": "\\\\?\\usb#vid_174f&pid_181d&mi_00#7&2e28cb65&0&0000#{65e8773d-8f56-11d0-a3b9-00a0c9223196}\\global"
  },
  "calling_selected_microphone_device": {
    "Microphone": "TEST"
  },
  "calling_selected_secondary_ringer_device": {
  },
  "calling_selected_speaker_device": {
    "Speaker": "TEST"
  },
  "calling_user_selected_devices": {
    "Microphone": "TEST",
    "Speaker": "TEST"
  }
    }"#;
        ms_teams_conf.to_string()
    }

    fn get_result() -> String {
        let ms_teams_conf = r#"{
  "calling_selected_camera_device": {
    "Camera": "\\\\?\\usb#vid_174f&pid_181d&mi_00#7&2e28cb65&0&0000#{65e8773d-8f56-11d0-a3b9-00a0c9223196}\\global"
  },
  "calling_selected_microphone_device": {
    "Microphone": "microphone"
  },
  "calling_selected_secondary_ringer_device": {
  },
  "calling_selected_speaker_device": {
    "Speaker": "speaker"
  },
  "calling_user_selected_devices": {
    "Microphone": "microphone",
    "Speaker": "speaker"
  }
    }"#;
        ms_teams_conf.to_string()
    }
}
