#ifndef _FANCY_TALK_H_
#define _FANCY_TALK_H_

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "bindings.h"

#define QUERY 0
#define RESPONSE 1

struct message_list {
    struct message_list *next;
    Package *message;
};

#endif // _FANCY_TALK_H_
