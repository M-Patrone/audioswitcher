use clap::Parser;
use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use inquire::Select;
use windows::{
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Media::{
            Audio::{
                DEVICE_STATE, DEVICE_STATE_ACTIVE, DEVICE_STATE_DISABLED, DEVICE_STATE_NOTPRESENT,
                IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, eCommunications, eConsole,
                eMultimedia, eRender,
            },
            MediaFoundation::{
                IMFActivate, IMFAttributes, MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ENDPOINT_ID,
                MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_GUID, MF_VERSION, MFCreateAttributes,
                MFEnumDeviceSources, MFSTARTUP_FULL, MFShutdown, MFStartup,
            },
        },
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED, CoCreateInstance,
            CoInitializeEx, CoTaskMemFree, CoUninitialize, STGM_READ,
        },
    },
    core::{GUID, PCWSTR, PWSTR, Result},
};

use log::{error, info};
use std::os::windows::ffi::OsStrExt;
use std::{ffi::OsStr, fmt};

use crate::args::{AudioSwitcherArgs, EntityType};
mod args;
mod teams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioType {
    Microphone,
    Speaker,
}

impl fmt::Display for AudioType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AudioType::Microphone => write!(f, "{}", "Microphone"),
            AudioType::Speaker => write!(f, "{}", "Speaker"),
        }
    }
}

#[derive(Debug, Clone)]
struct AudioDevice {
    pub device_id: String,
    pub device_friendly_name: String,
    pub audio_type: AudioType,
}

impl fmt::Display for AudioDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} ({})",
            self.audio_type, self.device_friendly_name, self.device_id
        )
    }
}

fn main() -> Result<()> {
    let args = AudioSwitcherArgs::parse();
    if args.verbose {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
    } else {
        env_logger::builder()
            .filter_level(log::LevelFilter::Error)
            .init();
    }
    execute_logic(args.entity_type)
}

fn execute_logic(entity_type: EntityType) -> Result<()> {
    let mut audio_devices: Vec<AudioDevice> = Vec::new();

    let res = display_microphone_devices();

    if res.is_ok() {
        audio_devices.append(&mut res.unwrap());
    }
    let res = display_speaker_devices(DEVICE_STATE_ACTIVE);

    if res.is_ok() {
        audio_devices.append(&mut res.unwrap());
    }
    let res = display_speaker_devices(DEVICE_STATE_NOTPRESENT);

    if res.is_ok() {
        audio_devices.append(&mut res.unwrap());
    }
    match entity_type {
        EntityType::Show => {
            display_devices(audio_devices);
        }
        EntityType::Select(e) => {
            start_user_process(audio_devices, e.change_windows_audio_device);
        }
        EntityType::Active => {
            get_active_devices();
        }
        EntityType::Set(e) => {
            if (e.change_windows_audio_device) {
                let res = set_speaker_windows_settings(e.speaker_device_id.clone());
            }
            if e.microphone_device_id.is_some() {
                teams::set_device_ms_teams(
                    e.speaker_device_id.clone(),
                    e.microphone_device_id.unwrap().clone(),
                );
            } else {
                teams::set_device_ms_teams(
                    e.speaker_device_id.clone(),
                    e.speaker_device_id.clone(),
                );
            }
        }
    }
    Ok(())
}

fn get_active_devices() -> Result<()> {
    let res = display_speaker_devices(DEVICE_STATE_ACTIVE);
    match res {
        Ok(devices) => {
            for device in devices {
                println!("Active device: {}", device);
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
    Ok(())
}

fn display_devices(audio_devices: Vec<AudioDevice>) {
    for item in audio_devices.iter() {
        println!(
            "{}: device_id '{}' with name '{}' ",
            item.audio_type, item.device_id, item.device_friendly_name
        );
    }
}

fn start_user_process(
    devices: Vec<AudioDevice>,
    change_windows_audio_device: bool,
) -> inquire::error::InquireResult<()> {
    let speakers = devices
        .clone()
        .into_iter()
        .filter(|f| f.audio_type == AudioType::Speaker)
        .collect();

    let microphones: Vec<AudioDevice> = devices
        .into_iter()
        .filter(|f| f.audio_type == AudioType::Microphone)
        .collect();

    let speaker_selected = Select::new("Choose the speaker", speakers).prompt()?;
    println!("selected speakers: {}", speaker_selected);

    let microphone_selected = Select::new("Choose the microphone", microphones).prompt()?;
    println!("selected microphone: {}", microphone_selected);

    if (change_windows_audio_device) {
        let res = set_speaker_windows_settings(speaker_selected.device_id.clone());
    }
    teams::set_device_ms_teams(speaker_selected.device_id, microphone_selected.device_id);

    Ok(())
}

fn display_microphone_devices() -> Result<Vec<AudioDevice>> {
    let mut microphone_devices: Vec<AudioDevice> = Vec::new();
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        MFStartup(MF_VERSION, MFSTARTUP_FULL);

        let mut attr_opt: Option<IMFAttributes> = None;
        MFCreateAttributes(&mut attr_opt, 1)?;

        let attr = attr_opt.unwrap();

        attr.SetGUID(
            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_GUID,
        )?;

        let mut ptr: *mut Option<IMFActivate> = std::ptr::null_mut();
        let mut count: u32 = 0;

        MFEnumDeviceSources(&attr, &mut ptr, &mut count)?;

        let slice = std::slice::from_raw_parts_mut(ptr, count as usize);

        for (i, slot) in slice.iter_mut().enumerate() {
            if let Some(activate) = slot.take() {
                let mut length: u32 = 0;
                let mut ppwszvalue = PWSTR::default();
                activate.GetAllocatedString(
                    &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                    &mut ppwszvalue,
                    &mut length,
                )?;

                let mut length_guid: u32 = 0;
                let mut ppwszvalue_guid = PWSTR::default();
                activate.GetAllocatedString(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ENDPOINT_ID,
                    &mut ppwszvalue_guid,
                    &mut length_guid,
                )?;

                let friendly_name = ppwszvalue
                    .to_string()
                    .unwrap_or_else(|_| "<not readable>".into());

                let device_guid = ppwszvalue_guid
                    .to_string()
                    .unwrap_or_else(|_| "<not readable>".into());

                microphone_devices.push(AudioDevice {
                    device_id: device_guid.clone(),
                    device_friendly_name: friendly_name.clone(),
                    audio_type: AudioType::Microphone,
                });

                info!("Friendly name: {friendly_name} with id {device_guid}");

                CoTaskMemFree(Some(ppwszvalue.0 as *mut _));
                CoTaskMemFree(Some(ppwszvalue_guid.0 as *mut _));
            }
        }

        CoTaskMemFree(Some(ptr as *mut _));

        MFShutdown();
        CoUninitialize();
    }
    Ok(microphone_devices)
}

fn display_speaker_devices(state: DEVICE_STATE) -> Result<Vec<AudioDevice>> {
    let mut speaker_devices: Vec<AudioDevice> = Vec::new();
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let instance: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let collection = instance.EnumAudioEndpoints(eRender, state)?;

        let count_devices = collection.GetCount()?;
        info!("could get {count_devices}");

        for i in 0..count_devices {
            let device: IMMDevice = collection.Item(i)?;
            let device_id = match device.GetId() {
                Ok(id) => Ok(id
                    .to_string()
                    .unwrap_or_else(|_| "<id-unicode-error>".into())),
                Err(e) => Err(e),
            };

            let store = match device.OpenPropertyStore(STGM_READ) {
                Ok(s) => s,
                Err(e) => {
                    error!("Error on getting property store: {}", e);
                    continue;
                }
            };

            let friendly_name = match store.GetValue(&PKEY_Device_FriendlyName) {
                Ok(v) => v.to_string(),
                Err(e) => {
                    error!("Error on getting property store: {}", e);
                    continue;
                }
            };

            if let Ok(id) = device_id.as_ref() {
                info!("Device {id} with name {friendly_name}");
                speaker_devices.push(AudioDevice {
                    device_id: id.clone(),
                    device_friendly_name: friendly_name,
                    audio_type: AudioType::Speaker,
                });
            }
        }
        CoUninitialize();
    }
    Ok(speaker_devices)
}

fn set_speaker_windows_settings(device_id: String) -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let policy: IPolicyConfig = unsafe { CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)? };

    let wide: Vec<u16> = OsStr::new(&device_id)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let dev_id = PCWSTR(wide.as_ptr());
    unsafe {
        policy.SetDefaultEndpoint(dev_id, eConsole)?;
        policy.SetDefaultEndpoint(dev_id, eMultimedia)?;
        policy.SetDefaultEndpoint(dev_id, eCommunications)?;
        drop(policy);
    }
    unsafe {
        CoUninitialize();
    }
    Ok(())
}
