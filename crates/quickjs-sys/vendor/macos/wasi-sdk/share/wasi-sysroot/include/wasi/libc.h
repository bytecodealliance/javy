#ifndef __wasi_libc_h
#define __wasi_libc_h

#include <__typedef_off_t.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Register the given pre-opened file descriptor under the given path.
///
/// This function does not take ownership of `prefix` (it makes its own copy).
int __wasilibc_register_preopened_fd(int fd, const char *prefix);

/// Renumber `fd` to `newfd`; similar to `dup2` but does a move rather than a
/// copy.
int __wasilibc_fd_renumber(int fd, int newfd);

/// Like `unlinkat`, but without depending on `__wasi_path_remove_directory`.
int __wasilibc_unlinkat(int fd, const char *path);

/// An `*at` version of rmdir.
int __wasilibc_rmdirat(int fd, const char *path);

/// Like `open`, but without the varargs in the signature.
int __wasilibc_open_nomode(const char *path, int oflag);

/// Like `openat`, but without the varargs in the signature.
int __wasilibc_openat_nomode(int fd, const char *path, int oflag);

/// Return the current file offset. Like `lseek(fd, 0, SEEK_CUR)`, but without
/// depending on `lseek`.
off_t __wasilibc_tell(int fd);

#ifdef __cplusplus
}
#endif

#endif
