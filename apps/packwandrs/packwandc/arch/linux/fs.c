/* pwfs backend: rooted filesystem access and inotify watching (packwandc.md 5.3).
 *
 * CONFINEMENT
 *
 * Every path this file touches is resolved with realpath and then checked to
 * lie beneath the resolved root. Resolution happens first on purpose: checking
 * the textual path before resolving it is the classic confinement bug, because
 * a symlink inside the root can point anywhere and a purely lexical check
 * cannot see it. Rejecting "..", which modules/pwfs/pwfs.c already does, is not
 * a substitute for this -- it is the cheap first filter in front of it.
 *
 * For writes the target usually does not exist yet, so it is the *parent*
 * directory that gets resolved and checked, and the basename is appended
 * afterwards. That keeps the guarantee without requiring the file to exist.
 *
 * ALLOCATION
 *
 * None. packwandc.md 3.4 confines malloc to kernel/arena.c and kernel/slab.c,
 * and scripts/gate-banned.sh enforces it, so every buffer here is a fixed-size
 * automatic with an explicit bound check in front of it.
 */

/* _GNU_SOURCE rather than a bare _POSIX_C_SOURCE, and not out of laziness:
 * glibc guards `struct dirent::d_type` and the DT_* constants behind
 * __USE_MISC, which _POSIX_C_SOURCE alone does not set. Under POSIX-only
 * defines this file would fail to compile on the field the watch walk depends
 * on. Feature-test macros must also precede every include or they are silently
 * ignored -- packwandc.md 7.1. */
#define _GNU_SOURCE 1

#include "packwandc/kernel/pwc_arch_fs.h"
#include "packwandc/kernel/pwc_error.h"

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h> /* realpath, mkstemp */
#include <string.h>
#include <sys/inotify.h>
#include <sys/stat.h>
#include <unistd.h>

enum {
    /* PATH_MAX on Linux. Named rather than repeated so a buffer cannot drift
     * out of step with the bound checked against it (packwandc.md 7.1). */
    PWC_FS_PATH_MAX = 4096,
    /* Depth cap for the watch walk. Recursion is banned (packwandc.md 7.5), so
     * the walk carries an explicit stack and the stack has to be bounded. */
    PWC_FS_WALK_MAX_DEPTH = 32,
    /* Upper bound on inotify watches per handle. A pathological tree must not
     * be able to exhaust the per-user inotify limit for the whole session. */
    PWC_FS_WATCH_MAX_DIRS = 4096,
    /* One read of the inotify fd. Sized for many coalesced events per call. */
    PWC_FS_EVENT_BUFFER = 8192,
    /* Mode for a published file. mkstemp makes the temporary 0600; the visible
     * result should look like any other content file. */
    PWC_FS_PUBLISH_MODE = 0644,
};

/* One level of the watch walk. Recursion is banned (packwandc.md 7.5), so the
 * traversal carries its own stack of these. */
typedef struct pwc_fs_walk_frame {
    DIR *dir;
    size_t len; /* length of the path prefix this frame owns */
} pwc_fs_walk_frame;

/* --- path construction and confinement ---------------------------------- */

static pwc_status pwc_fs_copy_bounded(char *out, size_t capacity, const uint8_t *bytes, size_t length) {
    if (length >= capacity) {
        return PWC_FAIL(PWC_EOVERFLOW, "arch/linux", "path component exceeds PATH_MAX");
    }
    if (length != 0u) {
        memcpy(out, bytes, length);
    }
    out[length] = '\0';
    return PWC_OK;
}

/* root + '/' + path, with every bound checked. */
static pwc_status
pwc_fs_join(const uint8_t *root, size_t root_len, const uint8_t *path, size_t path_len, char *out) {
    PWC_TRY(pwc_fs_copy_bounded(out, (size_t) PWC_FS_PATH_MAX, root, root_len));
    size_t used = strlen(out);
    while (used > 1u && out[used - 1u] == '/') {
        --used;
        out[used] = '\0';
    }
    if (path_len == 0u) {
        return PWC_OK;
    }
    /* +1 for the separator, +1 for the terminator. */
    if (used + 1u + path_len + 1u > (size_t) PWC_FS_PATH_MAX) {
        return PWC_FAIL(PWC_EOVERFLOW, "arch/linux", "joined path exceeds PATH_MAX");
    }
    out[used] = '/';
    memcpy(&out[used + 1u], path, path_len);
    out[used + 1u + path_len] = '\0';
    return PWC_OK;
}

/* True when `candidate` is `root` itself or lies beneath it. Both must already
 * be realpath output, or this is a lexical check pretending to be a real one. */
static bool pwc_fs_contains(const char *root, const char *candidate) {
    const size_t root_len = strlen(root);
    if (strncmp(candidate, root, root_len) != 0) {
        return false;
    }
    /* "/pack" must not be treated as containing "/packages": the character
     * after the prefix has to be a separator, or the two must be equal. */
    return candidate[root_len] == '\0' || candidate[root_len] == '/' || (root_len == 1u && root[0] == '/');
}

static pwc_status pwc_fs_resolve(const char *input, char *resolved) {
    if (realpath(input, resolved) == nullptr) {
        const int code = errno;
        return code == ENOENT ? PWC_FAIL_PLATFORM(PWC_ENOENT, "arch/linux", "path does not exist", code)
                              : PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "realpath failed", code);
    }
    return PWC_OK;
}

/* Resolve `root` and the joined target, and reject a target outside the root.
 * This is the single choke point every read path goes through. */
static pwc_status pwc_fs_resolve_within(const uint8_t *root,
                                        size_t root_len,
                                        const uint8_t *path,
                                        size_t path_len,
                                        char *out_resolved) {
    char joined[PWC_FS_PATH_MAX] = {0};
    char resolved_root[PWC_FS_PATH_MAX] = {0};
    char root_only[PWC_FS_PATH_MAX] = {0};

    PWC_TRY(pwc_fs_copy_bounded(root_only, (size_t) PWC_FS_PATH_MAX, root, root_len));
    PWC_TRY(pwc_fs_resolve(root_only, resolved_root));
    PWC_TRY(pwc_fs_join(root, root_len, path, path_len, joined));
    PWC_TRY(pwc_fs_resolve(joined, out_resolved));

    if (!pwc_fs_contains(resolved_root, out_resolved)) {
        /* Reached only when a symlink inside the root pointed outside it: the
         * lexical ".." check in modules/pwfs/pwfs.c cannot see this case. */
        return PWC_FAIL(PWC_EPERM, "arch/linux", "resolved path escapes the watch root");
    }
    return PWC_OK;
}

/* --- read --------------------------------------------------------------- */

pwc_status pwc_arch_fs_read(const uint8_t *root,
                            size_t root_len,
                            const uint8_t *path,
                            size_t path_len,
                            uint8_t *buffer,
                            size_t capacity,
                            size_t *out_len) {
    if (root == nullptr || buffer == nullptr || out_len == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_fs_read: null root, buffer or out_len");
    }
    char resolved[PWC_FS_PATH_MAX] = {0};
    PWC_TRY(pwc_fs_resolve_within(root, root_len, path, path_len, resolved));

    /* O_NOFOLLOW is belt-and-braces: the path is already fully resolved, so a
     * symlink here means it was swapped in between resolve and open. */
    const int fd = open(resolved, O_RDONLY | O_CLOEXEC | O_NOFOLLOW);
    if (fd < 0) {
        const int code = errno;
        return code == ENOENT ? PWC_FAIL_PLATFORM(PWC_ENOENT, "arch/linux", "file disappeared", code)
                              : PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "open for read failed", code);
    }

    size_t total = 0u;
    pwc_status status = PWC_OK;
    for (;;) {
        if (total == capacity) {
            /* One more byte would not fit: distinguish "exactly filled" from
             * "truncated" by attempting a final read. */
            uint8_t probe = 0u;
            const ssize_t extra = read(fd, &probe, 1u);
            if (extra > 0) {
                status = PWC_FAIL(PWC_EOVERFLOW, "arch/linux", "file is larger than the buffer");
            } else if (extra < 0) {
                status = PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "read failed", errno);
            }
            break;
        }
        const ssize_t got = read(fd, &buffer[total], capacity - total);
        if (got == 0) {
            break;
        }
        if (got < 0) {
            if (errno == EINTR) {
                continue;
            }
            status = PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "read failed", errno);
            break;
        }
        total += (size_t) got;
    }

    (void) close(fd);
    *out_len = total;
    return status;
}

/* --- atomic write ------------------------------------------------------- */

static pwc_status pwc_fs_write_all(int fd, const uint8_t *content, size_t content_len) {
    size_t total = 0u;
    while (total < content_len) {
        const ssize_t put = write(fd, &content[total], content_len - total);
        if (put < 0) {
            if (errno == EINTR) {
                continue;
            }
            return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "write to the temporary file failed", errno);
        }
        total += (size_t) put;
    }
    return PWC_OK;
}

/* fsync the directory itself, so the rename is durable and not just the bytes.
 * Skipping this is the classic "the file is empty after a power cut" bug: the
 * data reaches disk but the directory entry pointing at it does not. */
static pwc_status pwc_fs_sync_directory(const char *directory) {
    const int fd = open(directory, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (fd < 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "open of the parent directory failed", errno);
    }
    const int synced = fsync(fd);
    const int code = errno;
    (void) close(fd);
    return synced == 0
               ? PWC_OK
               : PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "fsync of the parent directory failed", code);
}

/* Write `content` to a fresh temporary in `parent`, make it durable, then move
 * it onto `base`. rename(2) within a directory is atomic, so a reader either
 * sees the whole previous file or the whole new one and never a partial write.
 *
 * The fsync before the rename is the half people remember; the fsync of the
 * directory afterwards is the half they forget, and without it the rename can
 * be lost even though the bytes survived. */
static pwc_status
pwc_fs_replace(const char *parent, const char *base, const uint8_t *content, size_t content_len) {
    static const char suffix[] = "/.pwc-XXXXXX";
    char temporary[PWC_FS_PATH_MAX] = {0};
    char target[PWC_FS_PATH_MAX] = {0};
    const size_t parent_len = strlen(parent);
    const size_t base_len = strlen(base);

    if (parent_len + sizeof(suffix) > (size_t) PWC_FS_PATH_MAX ||
        parent_len + 1u + base_len + 1u > (size_t) PWC_FS_PATH_MAX) {
        return PWC_FAIL(PWC_EOVERFLOW, "arch/linux", "write path exceeds PATH_MAX");
    }
    memcpy(temporary, parent, parent_len);
    memcpy(&temporary[parent_len], suffix, sizeof(suffix)); /* copies the NUL too */
    memcpy(target, parent, parent_len);
    target[parent_len] = '/';
    memcpy(&target[parent_len + 1u], base, base_len);
    target[parent_len + 1u + base_len] = '\0';

    const int fd = mkstemp(temporary);
    if (fd < 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "mkstemp failed", errno);
    }

    pwc_status status = pwc_fs_write_all(fd, content, content_len);
    if (status == PWC_OK && fsync(fd) != 0) {
        status = PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "fsync of the temporary file failed", errno);
    }
    /* mkstemp creates 0600. Pack files are ordinary content, so the published
     * file gets ordinary permissions rather than inheriting the private mode
     * the temporary needed. */
    if (status == PWC_OK && fchmod(fd, PWC_FS_PUBLISH_MODE) != 0) {
        status = PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "fchmod of the temporary file failed", errno);
    }
    if (close(fd) != 0 && status == PWC_OK) {
        status = PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "close of the temporary file failed", errno);
    }
    if (status == PWC_OK && rename(temporary, target) != 0) {
        status = PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "rename onto the target failed", errno);
    }
    if (status != PWC_OK) {
        /* Never leave a half-written temporary behind for the caller to trip
         * over on the next listing. */
        (void) unlink(temporary);
        return status;
    }
    return pwc_fs_sync_directory(parent);
}

/* Split "<dir>/<base>" into its two parts. Both outputs are PWC_FS_PATH_MAX. */
static pwc_status pwc_fs_split_parent(const char *joined, char *out_parent, char *out_base) {
    const char *const separator = strrchr(joined, '/');
    if (separator == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "joined path has no directory component");
    }
    const size_t parent_len = (size_t) (separator - joined);
    const size_t base_len = strlen(separator + 1);
    if (base_len == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "write target has no file name");
    }
    /* A target directly under "/" leaves a zero-length parent; that is "/". */
    PWC_TRY(pwc_fs_copy_bounded(
        out_parent, (size_t) PWC_FS_PATH_MAX, (const uint8_t *) joined, parent_len == 0u ? 1u : parent_len));
    if (parent_len == 0u) {
        out_parent[0] = '/';
        out_parent[1] = '\0';
    }
    return pwc_fs_copy_bounded(
        out_base, (size_t) PWC_FS_PATH_MAX, (const uint8_t *) (separator + 1), base_len);
}

pwc_status pwc_arch_fs_atomic_write(const uint8_t *root,
                                    size_t root_len,
                                    const uint8_t *path,
                                    size_t path_len,
                                    const uint8_t *content,
                                    size_t content_len) {
    if (root == nullptr || content == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_fs_atomic_write: null root or content");
    }
    char joined[PWC_FS_PATH_MAX] = {0};
    char parent[PWC_FS_PATH_MAX] = {0};
    char base[PWC_FS_PATH_MAX] = {0};
    char resolved_root[PWC_FS_PATH_MAX] = {0};
    char resolved_parent[PWC_FS_PATH_MAX] = {0};
    char root_only[PWC_FS_PATH_MAX] = {0};

    PWC_TRY(pwc_fs_copy_bounded(root_only, (size_t) PWC_FS_PATH_MAX, root, root_len));
    PWC_TRY(pwc_fs_resolve(root_only, resolved_root));
    PWC_TRY(pwc_fs_join(root, root_len, path, path_len, joined));
    PWC_TRY(pwc_fs_split_parent(joined, parent, base));

    /* The parent is resolved rather than the target: the target is usually
     * about to be created and realpath would fail on it. */
    PWC_TRY(pwc_fs_resolve(parent, resolved_parent));
    if (!pwc_fs_contains(resolved_root, resolved_parent)) {
        return PWC_FAIL(PWC_EPERM, "arch/linux", "resolved write target escapes the root");
    }
    return pwc_fs_replace(resolved_parent, base, content, content_len);
}

/* --- inotify watching --------------------------------------------------- */

static pwc_status pwc_fs_watch_add(int fd, const char *directory, uint32_t *count) {
    if (*count >= (uint32_t) PWC_FS_WATCH_MAX_DIRS) {
        return PWC_FAIL(PWC_EOVERFLOW, "arch/linux", "watch tree exceeds the per-handle directory cap");
    }
    const uint32_t mask = IN_CREATE | IN_DELETE | IN_MODIFY | IN_MOVED_FROM | IN_MOVED_TO | IN_ATTRIB |
                          IN_CLOSE_WRITE | IN_DELETE_SELF | IN_MOVE_SELF;
    if (inotify_add_watch(fd, directory, mask) < 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "inotify_add_watch failed", errno);
    }
    ++*count;
    return PWC_OK;
}

/* d_type is unset on some filesystems, so fall back to lstat. lstat and not
 * stat: a symlinked directory must not be descended into, both to avoid cycles
 * and because following one would watch a tree outside the root. */
static bool pwc_fs_entry_is_real_dir(const char *path, unsigned char d_type) {
    if (d_type == DT_DIR) {
        return true;
    }
    if (d_type != DT_UNKNOWN) {
        return false;
    }
    struct stat info = {0};
    return lstat(path, &info) == 0 && S_ISDIR(info.st_mode);
}

static bool pwc_fs_is_dot_entry(const char *name) {
    return name[0] == '.' && (name[1] == '\0' || (name[1] == '.' && name[2] == '\0'));
}

/* Rewrite `path` to "<prefix of length base>/<name>". False if it would not
 * fit, in which case `path` is left holding whatever it had. */
static bool pwc_fs_append_child(char *path, size_t base, const char *name) {
    const size_t name_len = strlen(name);
    if (base + 1u + name_len + 1u > (size_t) PWC_FS_PATH_MAX) {
        return false;
    }
    path[base] = '/';
    memcpy(&path[base + 1u], name, name_len);
    path[base + 1u + name_len] = '\0';
    return true;
}

/* Register an inotify watch on `path` and every directory beneath it.
 *
 * `path` is used as the working buffer for the whole walk and is left in an
 * unspecified state; callers must not read it afterwards.
 *
 * KNOWN LIMIT: directories created *after* this returns are not watched.
 * Catching them needs a watch-descriptor-to-path map so an IN_CREATE|IN_ISDIR
 * event can be turned back into a path, and that map needs storage this layer
 * has no allocator for -- it would have to come from a kernel slab wired
 * through boot. ReadDirectoryChangesW gets subtree recursion from the OS and
 * has no equivalent gap, so this is a real platform asymmetry rather than a
 * shared simplification. It is recorded in packwandc.md 5.3. */
static pwc_status pwc_fs_watch_tree(int fd, char *path) {
    pwc_fs_walk_frame stack[PWC_FS_WALK_MAX_DEPTH] = {0};
    uint32_t count = 0u;
    int depth = 0;

    PWC_TRY(pwc_fs_watch_add(fd, path, &count));
    stack[0].dir = opendir(path);
    if (stack[0].dir == nullptr) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "opendir of the watch root failed", errno);
    }
    stack[0].len = strlen(path);

    pwc_status status = PWC_OK;
    while (depth >= 0 && status == PWC_OK) {
        const struct dirent *const entry = readdir(stack[depth].dir);
        if (entry == nullptr) {
            (void) closedir(stack[depth].dir);
            --depth;
            continue;
        }
        if (pwc_fs_is_dot_entry(entry->d_name)) {
            continue;
        }
        /* Rebuilt from the frame's own prefix every time, so the tail left by
         * the previous entry never leaks into this one. */
        if (!pwc_fs_append_child(path, stack[depth].len, entry->d_name)) {
            continue;
        }
        if (!pwc_fs_entry_is_real_dir(path, entry->d_type)) {
            continue;
        }
        status = pwc_fs_watch_add(fd, path, &count);
        if (status != PWC_OK || depth + 1 >= PWC_FS_WALK_MAX_DEPTH) {
            continue;
        }
        DIR *const child = opendir(path);
        if (child != nullptr) {
            ++depth;
            stack[depth].dir = child;
            stack[depth].len = strlen(path);
        }
    }

    /* Only reached with frames still open when the loop broke on an error. */
    while (depth >= 0) {
        (void) closedir(stack[depth].dir);
        --depth;
    }
    return status;
}

pwc_status pwc_arch_fs_watch_open(const uint8_t *root, size_t root_len, uintptr_t *out_native) {
    if (root == nullptr || out_native == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_fs_watch_open: null root or out");
    }
    char resolved_root[PWC_FS_PATH_MAX] = {0};
    char root_only[PWC_FS_PATH_MAX] = {0};
    PWC_TRY(pwc_fs_copy_bounded(root_only, (size_t) PWC_FS_PATH_MAX, root, root_len));
    PWC_TRY(pwc_fs_resolve(root_only, resolved_root));

    /* Non-blocking: pwc_arch_fs_watch_read is a poll, and the blocking is the
     * job of pwc_wait rather than of a read that would pin a worker thread. */
    const int fd = inotify_init1(IN_NONBLOCK | IN_CLOEXEC);
    if (fd < 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "inotify_init1 failed", errno);
    }

    const pwc_status walked = pwc_fs_watch_tree(fd, resolved_root);
    if (walked != PWC_OK) {
        (void) close(fd);
        return walked;
    }
    /* Biased by one so the payload is never zero; the handle layer reads a
     * zero payload as "unset". */
    *out_native = (uintptr_t) ((uint32_t) fd + 1u);
    return PWC_OK;
}

pwc_status pwc_arch_fs_watch_read(uintptr_t native, size_t *out_events) {
    if (native == 0u || out_events == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_fs_watch_read: null watch or out");
    }
    const int fd = (int) ((uint32_t) native - 1u);

    /* Alignment matters: the buffer is cast to struct inotify_event, whose
     * first member is an int. A plain uint8_t array has alignment 1 and the
     * cast would be undefined -- and -Wcast-align would reject it. */
    _Alignas(struct inotify_event) uint8_t events[PWC_FS_EVENT_BUFFER] = {0};
    size_t counted = 0u;

    for (;;) {
        const ssize_t got = read(fd, events, sizeof(events));
        if (got < 0) {
            if (errno == EINTR) {
                continue;
            }
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
                break; /* drained */
            }
            return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "read of the inotify fd failed", errno);
        }
        if (got == 0) {
            break;
        }
        /* Coalescing happens here (packwandc.md 5.3): the caller is told how
         * many changes settled, not handed a storm of individual records. */
        size_t offset = 0u;
        while (offset + sizeof(struct inotify_event) <= (size_t) got) {
            const struct inotify_event *event = (const struct inotify_event *) (const void *) &events[offset];
            offset += sizeof(struct inotify_event) + (size_t) event->len;
            ++counted;
        }
    }

    *out_events = counted;
    return counted == 0u ? PWC_EAGAIN : PWC_OK;
}

pwc_status pwc_arch_fs_watch_close(uintptr_t native) {
    if (native == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_fs_watch_close: null watch");
    }
    const int fd = (int) ((uint32_t) native - 1u);
    /* Closing the inotify fd removes every watch registered on it, so there is
     * no per-directory teardown to get wrong. */
    if (close(fd) != 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "close of the inotify fd failed", errno);
    }
    return PWC_OK;
}
