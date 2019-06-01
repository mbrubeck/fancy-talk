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


Package *c_alloc_package(TALLOC_CTX *mem_ctx,
                         const char *query_str,
                         const char *payload_str,
                         int red, int green, int blue) {
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

    pkg->message_type = RESPONSE;
    pkg->red = red;
    pkg->green = green;
    pkg->blue = blue;
    return pkg;
}


struct message_list *create_messages(TALLOC_CTX *mem_ctx) {
    struct message_list *fallback;
    struct message_list *greeting;
    struct message_list *hamlet;
    struct message_list *farewell;
    struct message_list *exit;

    fallback = talloc_zero(mem_ctx, struct message_list);
    fallback->message = c_alloc_package(fallback, "fallback", "Not found!", 0xff, 0x00, 0x00);
    fallback->message->bold = true;
    fallback->message->blink = true;

    greeting = talloc_zero(mem_ctx, struct message_list);
    greeting->message = c_alloc_package(greeting, "greeting", "Hello, world!", 0xee, 0x66, 0x22);
    greeting->message->italic = true;

    hamlet = talloc_zero(mem_ctx, struct message_list);
    hamlet->message = c_alloc_package(hamlet, "hamlet", "Alas, poor Yorrick!", 0x00, 0x66, 0x66);
    hamlet->message->underlined = true;

    farewell = talloc_zero(mem_ctx, struct message_list);
    farewell->message = c_alloc_package(farewell, "farewell", "Time to sahay goooooodbyeeeeeee!!!!", 0x00, 0x22, 0x66);
    farewell->message->bold = true;

    exit = talloc_zero(mem_ctx, struct message_list);
    exit->message = c_alloc_package(exit, "exit", "Bye, bye.", 0x00, 0xcc, 0x00);
    exit->message->bold = true;
    exit->message->italic = true;

    fallback->next = greeting;
    greeting->next = hamlet;
    hamlet->next = farewell;
    farewell->next = exit;

    return fallback;
}


Package *lookup_message(const struct message_list *messages, const Package *query) {
    const struct message_list *curr = messages;
    while(curr) {
        if (strncmp(query->query, curr->message->query, query->query_len) == 0) {
            return curr->message;
        }
        curr = curr->next;
    }

    // Use fallback
    return messages->message;
}

struct server_ctx {
    Package *query;
    uint8_t *buffer;
};

int free_server_ctx(struct server_ctx *srv) {
    if (srv->query) {
        free_package(srv->query);
    }
    if (srv->buffer) {
        free_buffer(srv->buffer);
    }
};

int main(const int argc, const char** argv) {
    int sockfd;
    unsigned short portno = 6543;
    struct sockaddr_in server_addr;
    struct sockaddr_in client_addr;
    char *inbuf;
    uint8_t *outbuf;
    size_t buflen;
    size_t clientlen;
    struct message_list *messages;
    Package *query;
    Package *response;
    struct server_ctx *srv_ctx;
    TALLOC_CTX *mem_ctx;

    mem_ctx = talloc_new(NULL);
    messages = create_messages(mem_ctx);

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
        srv_ctx = talloc_zero(mem_ctx, struct server_ctx);
        talloc_set_destructor(srv_ctx, free_server_ctx);
        inbuf = talloc_size(srv_ctx, MAX_UDP_SIZE);
        buflen = recvfrom(sockfd, inbuf, MAX_UDP_SIZE, 0, (struct sockaddr *)&client_addr, (unsigned int *)&clientlen);
        if (buflen == 0) {
            goto done;
        }

        srv_ctx->query = decode_package((uint8_t *)inbuf, buflen);
        if (srv_ctx->query == NULL) {
            goto done;
        }

        response = lookup_message(messages, srv_ctx->query);

        encode_package(response, &srv_ctx->buffer, &buflen);

        buflen = sendto(sockfd, srv_ctx->buffer, buflen, 0, (struct sockaddr *)&client_addr, clientlen);
        if (strncmp("exit", srv_ctx->query->query, srv_ctx->query->query_len) == 0) {
            break;
        }

done:
        talloc_free(srv_ctx);
    }
    talloc_free(mem_ctx);
    return 0;
}
