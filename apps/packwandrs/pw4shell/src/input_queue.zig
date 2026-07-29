//! Bounded, allocation-free handoff queue for high-rate native input.
//! Producers never wait for a consumer: when full, a packet is counted and
//! dropped. The UI can report the count instead of stalling the window thread.
const std = @import("std");

pub fn Queue(comptime T: type, comptime capacity: usize) type {
    comptime {
        if (capacity < 2) @compileError("input queue capacity must be at least two");
    }
    return struct {
        const Self = @This();
        locked: std.atomic.Value(bool) = .init(false),
        items: [capacity]T = undefined,
        head: usize = 0,
        tail: usize = 0,
        count: usize = 0,
        dropped: u64 = 0,

        /// Non-blocking producer operation. `false` means the bounded queue
        /// was full and the caller should continue without retrying.
        pub fn push(self: *Self, item: T) bool {
            self.lock();
            defer self.unlock();
            if (self.count == capacity) {
                self.dropped +%= 1;
                return false;
            }
            self.items[self.head] = item;
            self.head = (self.head + 1) % capacity;
            self.count += 1;
            return true;
        }

        /// Non-blocking consumer operation.
        pub fn pop(self: *Self) ?T {
            self.lock();
            defer self.unlock();
            if (self.count == 0) return null;
            const item = self.items[self.tail];
            self.tail = (self.tail + 1) % capacity;
            self.count -= 1;
            return item;
        }

        fn lock(self: *Self) void {
            while (self.locked.swap(true, .acquire)) std.atomic.spinLoopHint();
        }
        fn unlock(self: *Self) void { self.locked.store(false, .release); }

        pub fn droppedCount(self: *Self) u64 {
            self.lock();
            defer self.unlock();
            return self.dropped;
        }
    };
}

test "queue is bounded and reports overflow" {
    var queue: Queue(u32, 2) = .{};
    try std.testing.expect(queue.push(10));
    try std.testing.expect(queue.push(20));
    try std.testing.expect(!queue.push(30));
    try std.testing.expectEqual(@as(u64, 1), queue.droppedCount());
    try std.testing.expectEqual(@as(?u32, 10), queue.pop());
    try std.testing.expectEqual(@as(?u32, 20), queue.pop());
    try std.testing.expectEqual(@as(?u32, null), queue.pop());
}