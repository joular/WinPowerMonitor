# <a href="https://www.noureddine.org/research/joular/"><img src="https://raw.githubusercontent.com/joular/.github/main/profile/joular.png" alt="Joular Project" width="64" /></a> Power Monitor for Windows

Power Monitor for Windows is a command line tool that read CPU's power consumption through RAPL's MSRs, and provides the power consumption of the CPU every second.

Power Monitor for Windows depends on the [Windows RAPL driver by Hubblo](https://github.com/hubblo-org/windows-rapl-driver), and therefore require installing the driver first, and runs on Intel or AMD CPUs (since Ryzen).

Power Monitor for Windows was initially developed as part of [JoularJX](https://github.com/joular/joularjx), but is now its separate project.

## :bulb: Usage

Just run the program in command line, or run the executable.
It'll print on the command line the power consumption of the CPU every second.

## :floppy_disk: Compilation

To compile the Power Monitor for Windows, open the project in Visual Studio and compile there.
Or open, Developer Command Prompt for VS (or Developer PowerShell for VS), and compile with this command:
```
msbuild.exe PowerMonitor.sln /property:Configuration=Release
```