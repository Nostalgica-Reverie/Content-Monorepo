// Arena and slab allocation primitives implemented in Zig.
//
// The layouts below mirror the public C ABI headers. Keeping this deliberately
// small avoids relying on Zig's in-progress C23 importer while the headers
// remain the single source of truth consumed by C and Rust callers.

const std = @import("std");

const Status = i32;
const ok: Status = 0;
const einval: Status = -1;
const enomem: Status = -5;
const eio: Status = -9;
const eoverflow: Status = -12;

const Arena = extern struct {
    memory: ?[*]u8,
    capacity: usize,
    used: usize,
};

const Slab = extern struct {
    memory: ?[*]u8,
    next: ?[*]u32,
    free_head: u32,
    capacity: u32,
    object_size: usize,
};

const slab_end = std.math.maxInt(u32);
const slab_allocated = slab_end - 1;

pub export fn pwc_arena_init(arena: *Arena, memory: ?*anyopaque, capacity: usize) callconv(.c) void {
    arena.memory = if (memory) |value| @ptrCast(value) else null;
    arena.capacity = capacity;
    arena.used = 0;
}

pub export fn pwc_arena_alloc(
    arena: ?*Arena,
    size: usize,
    alignment: usize,
    out: ?*?*anyopaque,
) callconv(.c) Status {
    const state = arena orelse return einval;
    const output = out orelse return einval;
    const memory = state.memory orelse return einval;
    if (size == 0 or alignment == 0 or !std.math.isPowerOfTwo(alignment)) return einval;

    const base = @intFromPtr(memory);
    const current = std.math.add(usize, base, state.used) catch return eoverflow;
    const aligned = std.mem.alignForward(usize, current, alignment);
    if (aligned < current) return eoverflow;
    const offset = aligned - base;
    if (offset > state.capacity or size > state.capacity - offset) return enomem;

    output.* = @ptrFromInt(aligned);
    state.used = offset + size;
    return ok;
}

pub export fn pwc_arena_reset(arena: ?*Arena) callconv(.c) void {
    if (arena) |state| state.used = 0;
}

pub export fn pwc_slab_init(
    slab: *Slab,
    memory: ?*anyopaque,
    next: ?[*]u32,
    capacity: u32,
    object_size: usize,
) callconv(.c) void {
    slab.memory = if (memory) |value| @ptrCast(value) else null;
    slab.next = next;
    slab.capacity = capacity;
    slab.object_size = object_size;
    slab.free_head = if (capacity == 0) slab_end else 0;

    var index: u32 = 0;
    while (index < capacity) : (index += 1) {
        next.?[index] = if (index + 1 < capacity) index + 1 else slab_end;
    }
}

pub export fn pwc_slab_alloc(slab: ?*Slab, out: ?*?*anyopaque) callconv(.c) Status {
    const state = slab orelse return einval;
    const output = out orelse return einval;
    const memory = state.memory orelse return einval;
    const next = state.next orelse return einval;
    if (state.object_size == 0) return einval;
    if (state.free_head == slab_end) return enomem;
    if (state.free_head >= state.capacity) return eio;

    const index = state.free_head;
    state.free_head = next[index];
    next[index] = slab_allocated;
    output.* = @ptrFromInt(@intFromPtr(memory) + @as(usize, index) * state.object_size);
    return ok;
}

pub export fn pwc_slab_free(slab: ?*Slab, object: ?*anyopaque) callconv(.c) Status {
    const state = slab orelse return einval;
    const value = object orelse return einval;
    const memory = state.memory orelse return einval;
    const next = state.next orelse return einval;
    if (state.object_size == 0) return einval;

    const base = @intFromPtr(memory);
    const address = @intFromPtr(value);
    if (address < base) return einval;
    const offset = address - base;
    if (offset % state.object_size != 0 or offset / state.object_size >= state.capacity) return einval;

    const index: u32 = @intCast(offset / state.object_size);
    if (next[index] != slab_allocated) return einval;
    next[index] = state.free_head;
    state.free_head = index;
    return ok;
}
