all: server.exe client.exe

server.exe: objs srcs/server.c
	cl /nologo /Foobjs/server.obj srcs/server.c wsock32.lib

client.exe: objs srcs/client.c
	cl /nologo /Foobjs/client.obj srcs/client.c wsock32.lib

objs:
	mkdir objs

clean:
	-rmdir /s /q objs client.exe server.exe 2> NUL

