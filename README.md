# statusline

A blazingly-fast successor to purplesyringa's [shell](https://github.com/purplesyringa/shell.git),
rewritten in Rust.

## Requirements

* Linux-compatible OS. Other OSes were not tested, but it will probably fail to run
* Bash, for the shell
* Git, for repo information
* Cargo, for installing and updating *or* Nix, for building

## Installation

0. Install rustup and stable rust.
   ```bash
   pacman -S rustup
   rustup toolchain add stable
   ```
   Visit [rustup.rs](https://rustup.rs/) if not on Arch-based distro to see how to install on other
   distros. You may need to run rustup installation with superuser rights

1. Install statusline from cargo
   ```bash
   cargo install statusline
   ```

2. Check if statusline is in path.
   ```bash
   statusline
   ```
   If "bash: statusline: command not found" is shown, check your `$PATH` and `~/.bashrc`, a folder
   where cargo install placed statusline binary should be there.

   If you wish to not add the directory to `$PATH`, you can just use full path instead of short one
   in `statusline env` below

3. Set preferred statusline icons' mode. Do not add this line for defaults!
   Available modes are:
   - `PS1_MODE=text`: use ASCII text instead of icons
   - `PS1_MODE=minimal`: use alternative icon set which is somewhat simpler but may be perplexing
   - otherwise: use default nerdfont icons
   ```bash
   echo 'export PS1_MODE=minimal' >> ~/.bashrc
   ```

4. Install the statusline to shell
   ```bash
   echo 'source <(statusline env)' >> ~/.bashrc
   ```

5. Apply changes immediately
   ```bash
   PS1_MODE=minimal source <(statusline env)
   ```

Don't forget to check `$PATH` and update from time to time.

## Nix way

Build and apply immediately:
```bash
nix-build --log-format multiline-with-logs && source <(result/bin/statusline env)
```

Use in NixOS:
```nix
programs.bash.promptInit = let
  statusline = pkgs.callPackage pins.statusline {};
in ''
  PS1_MODE=minimal source <(${statusline}/bin/statusline env)
'';
programs.bash.vteIntegration = false;
```

## Features

* __Colorized username__ and hostname to prevent confusion if this statusline is installed on
  more than one device --- especially if connecting over SSH. Red color is reserved for root user
* __Git status display__ which immediately display repo's "persistent" info along with current
  state (rebasing, merging, etc.), and almost immediately the status. In addition, part of
  the working directory path inside the most nested git repo is highlighted
* __Chassis icons__ to display the type of the host device and help further differentiate between
  devices
* __Build tools display__ to inform which commands can be executed to "make" the project in
  working directory. Makefile, ./configure, CMake, purplesyringa's ./jr, meson, nix and cargo
  are supported
* __Simplified homes__  to make path more informative. Current user's home becomes `~`,
  others' become `~username`. Some paths are ignored to not make any confusion
* __...and others__ like "readonly" display, exit code visualization, jobs count and prompt time

## How is this different from purplesyringa's shell?

* *Small*. It relies on a small amount of external libraries --- compared to a great lot of
  dependencies in "shell". Executable size is lesser than a megabyte with libc as its only
  dependency
* *Fast, even on slow devices*. I remember waiting more than 5 seconds before the prompt appeared
  with "shell". I have patched it to show at least something useful before "heavy" data arrives.
  But I was surprised that in less than a quarter of second I've got almost the same info with this
* *Maintained*. Only two people used purplesyringa's shell: me and her. After my disappointment
  with "shell"'s speed, I've started working on this project and she abandoned her one in favor of
  this one
* *Lesser bugs*. "kill: no process found", "why does pressing `<Tab>` make newer prompts broken",
  and some others --- are not present here by design
* *More icons*. Almost every icon was changed to more appropriate and clean one
* *Nicer git status*. Proper commit abbreviation, handling of "detached head", icons even here...
  I just had a sleepless night that day

## Command line options

```
statusline
    Display simple message "how to use". Useless, but may be used to check if statusline is in path
statusline env
    Print commands for `.bashrc`
statusline run [--return-code <return-code>] --jobs-count <jobs-count> [--elapsed-time <elapsed-time>] [--control-fd <control-fd>] [--mode <mode>]
    Print statusline as PS1 prompt. Is not meant to be invoked directly, however--
    Expects control-fd to exist, will kill itself when something passed to it
statusline colorize --what <str>
    Colorize <str> like hostname and username. Can be used to choose hostname which has the color
    you want
```

This should have some better formatting but I'm too lazy for this

