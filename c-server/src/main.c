#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <string.h>
#include <arpa/inet.h>
#include <talloc.h>

#include "fancy_talk.h"

#define MAX_UDP_SIZE 4096


Package *c_alloc_package(TALLOC_CTX *mem_ctx, const char *query_str, const char *payload_str) {
    Package *pkg;
    pkg = talloc_zero(mem_ctx, Package);

    if (query_str != NULL) {
        size_t len = strlen(query_str);
        pkg->query_len = len;
        pkg->query = talloc_strndup(pkg, query_str, len);
    }

    if (payload_str != NULL) {
        size_t len = strlen(payload_str);
        pkg->payload_len = len;
        pkg->payload = talloc_strndup(pkg, payload_str, len);
    }
    return pkg;
}


int main(const int argc, const char** argv) {
    int sockfd;
    unsigned short portno = 6543;
    struct sockaddr_in server_addr;
    struct sockaddr_in client_addr;
    char *inbuf;
    uint8_t *outbuf;
    size_t buflen;
    size_t clientlen;
    Package *query;
    Package *response;
    TALLOC_CTX *mem_ctx;

    mem_ctx = talloc_new(NULL);
    response = c_alloc_package(mem_ctx, NULL, "Not found!");

    response->message_type = RESPONSE;
    response->bold = true;
    response->blink = true;
    response->red = 0xff;

    sockfd = socket(AF_INET, SOCK_DGRAM, 0);
    if (sockfd < 0) {
        printf("Error opening socket.\n");
        exit(1);
    }

    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    server_addr.sin_addr.s_addr = htonl(INADDR_ANY);
    server_addr.sin_port = htons(portno);

    if (bind(sockfd, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        printf("Error binding to socket\n");
        exit(1);
    }

    clientlen = sizeof(client_addr);

    while(1) {
        inbuf = talloc_size(mem_ctx, MAX_UDP_SIZE);
        buflen = recvfrom(sockfd, inbuf, MAX_UDP_SIZE, 0, (struct sockaddr *)&client_addr, (unsigned int *)&clientlen);
        if (buflen == 0) {
            talloc_free(inbuf);
            continue;
        }

        query = decode_package((uint8_t *)inbuf, buflen);
        if (query == NULL) {
            talloc_free(inbuf);
            continue;
        }

        free_package(query);
        talloc_free(inbuf);

        // TODO: look up messages
        encode_package(response, &outbuf, &buflen);


        buflen = sendto(sockfd, outbuf, buflen, 0, (struct sockaddr *)&client_addr, clientlen);
        free_buffer(outbuf);

    }
    talloc_free(response);
    return 0;
}
