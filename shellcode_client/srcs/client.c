#include <stdlib.h>
#include <stdio.h>
#include <winsock.h>

int
main(void)
{
	SOCKET sock;
	WSADATA wsaData;
	unsigned int len;
	unsigned char *buf;
	unsigned int off = 0;
	struct sockaddr_in sockaddr = { 0 };

	// Initialize WSA
	if(WSAStartup(MAKEWORD(2, 0), &wsaData)) {
		fprintf(stderr, "WSAStartup() error : %d", WSAGetLastError());
		return -1;
	}

	// Create TCP socket
	sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
	if(sock == INVALID_SOCKET) {
		fprintf(stderr, "socket() error : %d", WSAGetLastError());
		return -1;
	}

	sockaddr.sin_family = AF_INET;
	sockaddr.sin_addr.s_addr = inet_addr("192.168.1.2");
	sockaddr.sin_port = htons(1234);

	// Connect to the socket
	if(connect(sock, (const struct sockaddr*)&sockaddr, sizeof(sockaddr)) == SOCKET_ERROR) {
		fprintf(stderr, "connect() error : %d", WSAGetLastError());
		return -1;
	}

	// Read the payload length
	if(recv(sock, (char*)&len, sizeof(len), 0) != sizeof(len)) {
		fprintf(stderr, "recv() error : %d", WSAGetLastError());
		return -1;
	}

	// Read the payload
	buf = malloc(len);
	if(!buf) {
		perror("malloc() error ");
		return -1;
	}

	while(off < len) {
		int bread;
		unsigned int remain = len - off;
		bread = recv(sock, buf + off, remain, 0);
		if(bread <= 0) {
			fprintf(stderr, "recv(pl) error : %d", WSAGetLastError());
			return -1;
		}

		off += bread;
	}

	printf("Read everything %u\n", off);

	// FELF0001 + u64 entry + u64 base
	if(len < 3 * 8) {
		fprintf(stderr, "Invalid FELF\n");
		return -1;
	}

	{
		char *ptr = buf;
		unsigned int entry, base, hi, end;

		if(memcmp(ptr, "FELF0001", 8)) {
			fprintf(stderr, "Missing FELF header\n");
			return -1;
		}
		ptr += 8;

		entry = *((unsigned int*)ptr)++;
		hi = *((unsigned int*)ptr)++;
		if(hi) {
			fprintf(stderr, "Unhandled 64-bit address\n");
			return -1;
		}

		base = *((unsigned int*)ptr)++;
		hi = *((unsigned int*)ptr)++;
		if(hi) {
			fprintf(stderr, "Unhandled 64-bit address\n");
			return -1;
		}

		end = base + (len - 3 * 8);
		printf("Loading at %x-%x (%x) entry %x\n", base, end, end - base, entry);

		{
			unsigned int align_base = base & ~0xffff;
			unsigned int align_end  = (end + 0xffff) & ~0xffff;
			char *alloc = VirtualAlloc((void*)align_base,
				align_end - align_base, MEM_COMMIT | MEM_RESERVE,
				PAGE_EXECUTE_READWRITE);
			printf("Alloc attempt %x-%x (%x) | Got %p\n",
				align_base, align_end, align_end - align_base, alloc);
			if(alloc != (void*)align_base) {
				fprintf(stderr, "VirtualAlloc() error : %d\n", GetLastError());
				return -1;
			}

			// Copy in the code
			memcpy((void*)base, ptr, end - base);
		}

		// Jump to the entry
		((void (*)(SOCKET))entry)(sock);
	}

	return 0;
}
