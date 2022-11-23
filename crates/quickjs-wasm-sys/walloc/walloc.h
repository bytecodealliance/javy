#ifndef __WALLOC_H__
#define __WALLOC_H__

typedef __SIZE_TYPE__ size_t;
typedef unsigned char *uint8_t;

uint8_t * wmalloc(size_t size, size_t alignment);
void wfree(uint8_t *p);

#endif /* __WALLOC_H__ */
