# Audioswitcher

AudioSwitcher is a small Windows utility that allows you to switch your default audio devices (both speakers and microphones) via command line.
It also attempts to update the active device within Microsoft Teams automatically.

This tool was originally created to simplify switching audio outputs using an Elgato Stream Deck button, ideal for users who frequently switch between headsets, speakers, or other audio devices.

**Why not a PowerShell-Skript?**

Two simple reasons:

- Learning experience: This project was a way to explore something new beyond PowerShell.
- No extra dependencies: PowerShell solutions often require additional [downloads or module](https://stackoverflow.com/questions/66824212/change-default-windows-sound-with-powershell)

You can also read the related [Microsoft Teams issue discussion](https://learn.microsoft.com/en-us/answers/questions/4420831/how-to-set-computer-audio-as-the-default-one-in-te) for more context.

## Requirements

- Works only on Windows (tested on Windows 10/11).
- Not supported on macOS or Linux.
- You should have Microsoft Teams installed if you want to use the Teams integration.

## Installation

1. Download the latest `AudioSwitcher.exe` from the Releases page.
2. Place the executable in a folder of your choice.
3. (Optional) Create a batch file that calls the .exe with your desired command, perfect for binding to a Stream Deck button.

Example batch file:

```
audioswitcher.exe set  "{0.0.0.00000000}.{6608c087-c1c8-42fv2-b62d-ce23hfajk7229}" "{0.0.1.00000000}.{e934ftacd-c4b5-4ce1-a239-fc3abd93247dh9}"
```

## Available commands

Here’s a list of all available commands and options:

```
Commands:
  show    Shows all audio devices (including speaker and microphone)
  select  Set with dropdown (useful if you don't know the device id)
  active  Get the active speaker and microphone on windows
  set     Set the device which you want to display
  help    Print this message or the help of the given subcommand(s)

Options:
      --verbose  enable extensive logging
  -h, --help     Print help
  -V, --version  Print version
```

## Errors

If an error occurs, the application creates a backup of the `cmd_settings.json` file and saves it as `cmd_settings_backup.json` in the same directory as the executable.

If you experience an error in Microsoft Teams or the application corrupts the settings file, you can manually restore the configuration by copying the content of the backup file into `cmd_settings.json`.

As of October 15, 2025, the Microsoft Teams settings file is located in the following folder:
`%LocalAppData%\Packages\MSTeams_8wekyb3d8bbwe\LocalCache\Microsoft\MSTeams`
