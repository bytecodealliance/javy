#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <assert.h>

#ifdef DEBUG
void *actual_in;
void *actual_out;
#endif

void* wmalloc_unaligned(size_t size);
void wfree_unaligned(void *ptr);

int 
is_aligned (void *p, size_t alignment)
{
  return ((uint64_t)p % alignment == 0);
}

void *
make_aligned(void *p, size_t alignment)
{
  p += alignment - 1;
  p -= ((uint64_t)p % alignment);
  return p;
}

uint8_t *
wmalloc(size_t size, size_t alignment)
{
  const unsigned offset_field_size = 2;

  assert(size > 0);
  
  // Increase the size of the allocation by so much that,
  // even if the allocator is at best byte-aligned,
  // a) it straddles a boundary of two pages, each of size `alignment`; and
  // b) and is big enough to accomodate the 2-byte offset field.
  if (alignment < 4) {
    // So that the offset field is aligned to two bytes
    alignment = 4;
  }
  size += alignment;
  size += offset_field_size;

  // Perform the allocation
  void *p = wmalloc_unaligned(size);
  assert(p != 0);
#ifdef DEBUG
  actual_in = p;
#endif

  // Realign p
  void *p_aligned;
  if (is_aligned(p, alignment)) {
    p_aligned = p + alignment;
  } else {
    p_aligned = make_aligned(p, alignment);
  }

  // Compute the offset field and store it
  uint16_t *offset_field_p = (uint16_t*)p_aligned;
  --offset_field_p;
  *offset_field_p = p_aligned - p;

#ifdef DEBUG
  printf("Returning     %p for actual address %p; allocated %08zu bytes\n", p_aligned, p, size);
#endif

  // Return the aligned pointer
  return (uint8_t*)p_aligned;
}

void 
wfree(uint8_t *p_)
{
  void *p = p_;

  // Calculate the address of the offset field
  uint16_t* offset_field_p = (uint16_t*)p;
  offset_field_p -= 1;

  // Get the value of the offset field
  uint16_t offset = *offset_field_p;

  // Calculate the address of the allocated block
  void *p_actual = p - offset;

#ifdef DEBUG
  printf("Free deriving %p from given address %p\n", p_actual, p);
#endif

  // Free the allocated block
  wfree_unaligned(p_actual);

#ifdef DEBUG
  actual_out = p_actual;
#endif
}

static int
unit_test()
{
  for (unsigned size = 1; size <= (1<<20); ++size) {
    for (unsigned alignment = 1; alignment <= (1<<12); alignment <<= 1) {
#ifdef DEBUG
      printf("size = %u, alignment = %u\n", size, alignment);
#endif
      void *p = wmalloc(size, alignment);
      wfree(p);
#ifdef DEBUG
      assert(actual_in == actual_out);
#endif
    }
  }
  return 0;
}
