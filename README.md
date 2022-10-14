# Logitech Litra Glow Control

For some bizarre reason, Logitech in their infinite wisdom has not
released any information how to control the *Litra Glow* video light
from anything else but the Logitech G-Hub. 

Now this works perfectly fine, however my mouse has only so many buttons
and I have many fine non-logitech keyboards lacking the required G-Keys 
to control the light forcing me to completely stretch my arm all the way
to the top of my monitor to manually turn on the light or to navigate
clickety-clackety-click with the mouse to the right page in the G-Hub to 
turn on the light. 

Clearly there is room for improvement and a quick google found some solutions:

 - [litra-driver](https://github.com/kharyam/litra-driver) for python, also on PyPI
 - [go-litra-driver](https://github.com/derickr/go-litra-driver) for Go
 - [Controlling the Logitech Litra on MacOS](https://ultracrepidarian.phfactor.net/tag/mac/)

Since my main meeting PC is Windows it would be nice if it worked there, but all
the above solutions are for Linux and MacOS. I tried to run them but I crashed and
burned. The python version complained about missing backends and the Go version
assumes *gcc* is available, which I could work around. The article on controlling
the Litra on MacOS uses *hidapitester* and actually works well also under Windows.

First hurdle is that WSL2 has no USB access, there are workarounds for that 
which involved proxying USB packets over IP and shuttling between host and WSL2 
environment and thoroughly scared me. So using the Linux solutions stopped also there.

Trying to find a Windows solution stranded in the effective SEO of all kind of outfits
trying to sell me stuff that I lost my patience long before I would realistically find 
a solution.

So let's chalk this up to market research.

# Requirements

Since no ready-made solution fell in my lap, and deploying what I found would be a 
_project_ so good to clarify requirements.

I want:
- to control the light with a keyboard shortcut
- it to be a simple command-line app, e.g. `litra-control on`
- it has to work on Linux, Windows (and Mac eventually, I gave mine to my daughter)
- it has to be consistent in Lin/Win/Mac and posh/bash/zsh (which rules out shell scripts)
- it has to be able to live in *~/.local/bin* as I do not want to fiddle with my PATH variable
- it has to have minimal dependencies and not break when Python updates
- it should be able to keep working for years without surprises
- it should be a learning experience

This leads me to some design choices 
- it has to be a compiled binary (to reduce dependency on runtimes) or equivalent
- it needs a good argument parser library (not a problem, ubiquitous now)
- it needs USB/HID access (this could be tricky due to quality differences)
- use environment variables for configuration to avoid hunting files
- it needs to have a meaningful name so I can remember what it does when I clean my *bin*.

Go would be ideal to quickly build this simple thing as it has great text UI libraries and arg
parsers, however I did not want to deal with the gcc dependency (or with the C compilation 
errors in MSys2 environment ).

So this is a good opportunity to brush off Rust. So I started with a single big file, split it
up when it became unwieldy. Learned more about USB and HID than I care to know and after many 
refactorings replacing the USB library multiple times I have something close to the requirements.

# Installation

To keep it simple

    cargo build --release
    cp target/release/litra-command ~/.local/bin

## Linux permissions

To access the device we need to tell the OS to put the right permissions on the device to allow
access without using *sudo* because that would be not simple to use.

For Arch and similar distro embracing the systemd wya we can add a
file */etc/udev/rules.d/71-litra-glow* with the content

    SUBSYSTEMS=="usb", ATTRS{idVendor}=="046d", ATTRS{idProduct}=="c900", MODE:="0660", TAG+="uaccess"

to allow user access to the file.

For Ubuntu and other debian based system the group *plugdev* is used to give people access to USG
and other hotplug devices:

   SUBSYSTEMS=="usb", ATTRS{idVendor}=="046d", ATTRS{idProduct}=="c900", MODE:="0660", GROUP="plugdev"

Of course you have to be in the *plugdev* group for this to work, if not:

   sudo usermod <yourlogin> -a -G plugdev

and logout/login again to make it stick.

Not recommended but when all else fails :

   SUBSYSTEMS=="usb", ATTRS{idVendor}=="046d", ATTRS{idProduct}=="c900", MODE:="0666"

This is insecure and with the current energy prices you do not want hackers to turnn on your video 
light sneakily. On the other hand al other USB manufacturers seem to prefer this approach as I find
many of these rules installed for other devices. In any case I recommend you figure out the right way
instead of the easy/insecure way.

# Usage

## Preparation

Because enumerating USB devices can be slow (looking at you, Windows) and guessing which device actually
controls the light is nontrivial (based on my vast experience with a sample size of 1 device) I rely on 
a *LITRA_PATH* variable to directly connect to the device. This also would kind of solve the issue if I 
would ever have more than one of those things connected to my PC.

The *init* command helps selecting the paths as they can only be found by asking the USB subsystem. 

    > litra-control init
    Configuration: LitraConfig { vendor_id: 1133, product_id: 51456, path: "\\\\?\\HID#VID_046D&PID_C900&Col02#a&8fac6bd&0&0001#{4d1e55b2-f16f-11cf-88cb-001111000030}" }
    Scanning USB devices. This might take a few seconds.

    HID Path : \\?\HID#VID_046D&PID_C900&Col01#a&8fac6bd&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}
    HID Path : \\?\HID#VID_046D&PID_C900&Col02#a&8fac6bd&0&0001#{4d1e55b2-f16f-11cf-88cb-001111000030}

    Set the environment variable LITRA_PATH to one of these values to avoid enumeration of devices.
    Unfortunately it is hard to specify which one is the right one as it depends on the platform.
    On Windows there are 2 per light and the second one is the one you want.

This command lists the *Litra Glow* devices it finds (2 on Windows, 1 on Linux). On Windows the second
worked for me, on Linux there is only 1. Copy-paste it in an environment variable. If it is in your 
*.bashrc* or powershell profile then double check that any weird characters are properly escaped.

# Usage

    $ litra-control on

turns it on

    $ litra-control off 

turns the light off. 

There is support for brightness and color temperature:

    $ litra-control brightness <0-100>

Sets the brightness as a percentage. 

    $ litra-control temperature <2700-6500>

Sets the temperature in Kelvin.

# Integration with the Desktop 

I already run [AutoHotKey](https://www.autohotkey.com/) to remap keys on my 
keyboard so that is the obvious choice to control the lamp with a keystroke.

    ;; Control the Litra Glow with Win-Alt-L
    LitraState = 0
    #!l::
    if (LitraState = 0) {
        Run, "C:\Users\%A_UserName%\.local\bin\litra-control.exe" on, , Hide
        LitraState = 1
    } else {
        Run, "C:\Users\%A_UserName%\.local\bin\litra-control.exe" off, , Hide
        LitraState = 0
    }
    return

I made it a toggle button in the *AutoHotKey* because it made most sense to me :
when using the keyboard I expect interactivity, but when running commands I find
it easier to reason if they always have the same postcondition and not have hidden
state. (I am probably overthinking this.)


# Integration in the terminal

While I prefer a meaningful name for the command, I do prefer a short form for
typing on the terminal. 

I added to my *~/.zshrc* (should be the same for *~/.bashrc* )

    # Map the litra-control script to an easy to type alias
    alias lc=litra-control

(on reflection I should have added that to *~/.profile* as that is shared between
bash and zsh, but let's keep that for future improvement. And yes, I realize I could
have fixed that faster that documenting this here.)

In the WSL2 terminal I need to make the command appear on the path (I do not have
my windows bin folder exposed in my Linux environment):

    $ ln -s ~/win/.local/bin/litra-control.exe ~/.local/bin/litra-control

(*~/win is a symbolic link to my windows home folder, e.g. /mnt/c/Users/<UserName>)

For powershell I added the following lines to *$PROFILE* :

    # Assign litra-control to alias lc
    New-Item -Path Alias:lc -Value c:\Users\PeterTillemans\.local\bin\litra-control.exe

