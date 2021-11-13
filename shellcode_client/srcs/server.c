#include <stdlib.h>
#include <stdio.h>
#include <winsock.h>

int
main(void)
{
	SOCKET sock;
	WSADATA wsaData;
	struct sockaddr_in sockaddr = { 0 };

	// Initialize WSA
	if(WSAStartup(MAKEWORD(2, 0), &wsaData)) {
		fprintf(stderr, "WSAStartup() error : %d\n", WSAGetLastError());
		return -1;
	}

	// Create TCP socket
	sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
	if(sock == INVALID_SOCKET) {
		fprintf(stderr, "socket() error : %d\n", WSAGetLastError());
		return -1;
	}

	sockaddr.sin_family = AF_INET;
	sockaddr.sin_addr.s_addr = inet_addr("0.0.0.0");
	sockaddr.sin_port = htons(42069);

	if(bind(sock, (const struct sockaddr*)&sockaddr, sizeof(sockaddr)) == SOCKET_ERROR) {
		fprintf(stderr, "bind() error : %d\n", WSAGetLastError());
		return -1;
	}

	// Listen for connections
	if(listen(sock, 5) == SOCKET_ERROR) {
		fprintf(stderr, "listen() error : %d\n", WSAGetLastError());
		return -1;
	}

	// Wait for a client
	for( ; ; ) {
		STARTUPINFO si = { 0 };
		PROCESS_INFORMATION pi = { 0 };
		SOCKET client = accept(sock, NULL, NULL);

		// Upon getting a TCP connection, just start
		// a separate client process. This way the
		// client can crash and burn and this server
		// stays running just fine.
		CreateProcess(
			"client.exe",
			NULL,
			NULL,
			NULL,
			FALSE,
			CREATE_NEW_CONSOLE,
			NULL,
			NULL,
			&si,
			&pi
		);

		// We don't even transfer data, we just care about
		// the connection kicking off a client.
		closesocket(client);
	}

	return 0;
}
