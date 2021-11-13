# Summary

This is a project which allows us to run Rust "shellcode" in a MIPS
environment on NT 4.0.

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

# Felserv

`felfserv` is a server for FELF files. You can find the FELF converter at
(elfloader)[https://github.com/gamozolabs/elfloader]. You need to install this
to your path as the `Makefile` invokes `elfloader` to convert the MIPS ELF into
MIPS shellcode in the FELF file format.

`felfserv` simply runs like `felfserv 0.0.0.0:1234 ./out.felf`. It will listen
to connections on IP and port you specified, and when connected to will
serve up the specific felf over a very basic protocol. This is what the
`client.exe` in the guest communicates with to download the Rust shellcode.

