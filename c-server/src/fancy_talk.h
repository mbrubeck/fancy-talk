#ifndef _FANCY_TALK_H_
#define _FANCY_TALK_H_

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#define QUERY 0
#define RESPONSE 1

typedef struct package {
	uint16_t id;
	uint8_t message_type;
	bool bold;
	bool italic;
	bool underlined;
	bool blink;
	uint8_t red;
	uint8_t green;
	uint8_t blue;
	size_t query_len;
	char *query;
	size_t payload_len;
	char *payload;
} Package;

Package *decode_package(const uint8_t* buffer, size_t len);
int encode_package(const Package *package, uint8_t **buffer, size_t *len);
void free_package(Package *package);
void free_buffer(uint8_t *buffer);


#endif // _FANCY_TALK_H_
