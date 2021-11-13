# Summary

This is a project which allows us to run Rust "shellcode" in a MIPS
environment on NT 4.0.

# TL;DR

## Setup NT

Install NT 4.0 MIPS in QEMU using the command you see in `qemu/run.sh`.

### Create disk and run system

```
qemu-img create –f qcow2 nt4.disk 2G
./qemu/run.sh
```

### Setup system so you can access CD

```
Run Setup > Initialize system > Set default configuration > (choose your res)
    > Floppy 3.5
    > Second floppy: No
    > SCSI host ID 7
```

### Setup ethernet address so network works in Windows

```
Run Setup > Initialize system > Set ethernet address
    > Pick an address (MUST BE A UNICAST MAC ADDRESS OR WINDOWS GETS MAD)
    > I used be2d08345673 with great success
```

### Boot partition

You must configure a small boot partition for the bootloader

Go to run program:

```
cd:\mips\arcinst
```

A 5 MiB partition will do

### Install Windows

```
cd:\mips\setupldr
```

### Configure time

The time in Windows doesn't persist, set it inside Windows to something
reasonable otherwise you'll get weird errors and `cl.exe` will not work so
you won't be able to compile anything.

## Use the tool

Deploy `server.exe` and `client.exe` to the system, then run `server.exe`
inside QEMU.

Install `felfserv` to your path `cd felfserv && cargo install --path .`

Run `felfserv` (supplies code to guest over network and stdout prints from Rust
running in guest) `felfserv 0.0.0.0:1234 ./out.felf`

Run `make` to build and deploy to MIPS!

Optionally run `cargo watch -- make` to get your code to re-deploy and run
every time you change the Rust project.

# Toolchain

To use this you need to copy the `shellcode_client` into a MIPS guest build
and run `server.exe` (included without any backdoors).

The server binds to `0.0.0.0:42069` and waits for a TCP connection. Upon a TCP
connection the server inside the guest will launch `client.exe` in the same
directory in a new process, which will then connect to the host via
`192.168.1.2:1234` to download the payload.

The reason we have `client.exe` in a separate process is so that we can crash
it without problems on the server. This provides a seamless development
experience when you use something like `cargo watch -- make` which will
automatically use `nc` to tickle the server, causing the client to connect
to the hosted `felfserv` which then causes the payload to execute in the guest.

The comms from the guest are sent to the `felfserv` over the socket that was
used to download the shellcode.

# Felfserv

`felfserv` is a server for FELF files. You can find the FELF converter at
[elfloader](https://github.com/gamozolabs/elfloader). You need to install this
to your path as the `Makefile` invokes `elfloader` to convert the MIPS ELF into
MIPS shellcode in the FELF file format.

`felfserv` simply runs like `felfserv 0.0.0.0:1234 ./out.felf`. It will listen
to connections on IP and port you specified, and when connected to will
serve up the specific felf over a very basic protocol. This is what the
`client.exe` in the guest communicates with to download the Rust shellcode.

