//! Safe parser for pw4shell source.  This layer performs no process spawning.
const std = @import("std");

pub const Error = error{
    UnterminatedQuote,
    InvalidEscape,
    UnexpectedNewline,
    InvalidFlag,
    DuplicateFlag,
    MissingExpansionName,
    UnterminatedExpansion,
    InvalidCollectionIndex,
    InvalidErrorMarker,
};

pub const Options = struct {
    /// Called for `$name` and `${name}` in double-quoted text. Missing values
    /// expand to an empty string, matching ordinary shell interpolation.
    lookup: ?*const fn (ctx: ?*anyopaque, name: []const u8) ?[]const u8 = null,
    lookup_context: ?*anyopaque = null,
};

pub const Value = union(enum) {
    text: []u8,
    error_marker,
    collection: CollectionRef,

    pub fn deinit(self: *Value, allocator: std.mem.Allocator) void {
        switch (self.*) {
            .text => |value| allocator.free(value),
            .collection => |value| allocator.free(value.name),
            .error_marker => {},
        }
    }
};

pub const CollectionRef = struct { name: []u8, index: usize };

pub const Command = struct {
    words: std.ArrayList(Value) = .empty,
    semantic_flags: std.ArrayList(u8) = .empty,
    execution_flags: std.ArrayList(u8) = .empty,

    pub fn deinit(self: *Command, allocator: std.mem.Allocator) void {
        for (self.words.items) |*word| word.deinit(allocator);
        self.words.deinit(allocator);
        self.semantic_flags.deinit(allocator);
        self.execution_flags.deinit(allocator);
    }

    pub fn format(self: Command, writer: *std.Io.Writer) !void {
        for (self.words.items, 0..) |word, i| {
            if (i != 0) try writer.writeAll(" ");
            switch (word) {
                .text => |text| try writer.print("{s}", .{text}),
                .error_marker => try writer.writeAll("$"),
                .collection => |ref| try writer.print("{s}[{d}]", .{ ref.name, ref.index }),
            }
        }
        for (self.semantic_flags.items) |flag| try writer.print(" --{c}", .{flag});
        for (self.execution_flags.items) |flag| try writer.print(" -{c}", .{flag});
    }
};

pub const Script = struct {
    allocator: std.mem.Allocator,
    commands: std.ArrayList(Command) = .empty,
    pub fn deinit(self: *Script) void {
        for (self.commands.items) |*command| command.deinit(self.allocator);
        self.commands.deinit(self.allocator);
    }
};

const Parser = struct {
    allocator: std.mem.Allocator,
    source: []const u8,
    options: Options,
    cursor: usize = 0,
    command: Command = .{},
    script: Script,
    at_line_start: bool = true,

    fn run(self: *Parser) !Script {
        errdefer self.command.deinit(self.allocator);
        errdefer self.script.deinit();
        while (self.cursor < self.source.len) {
            self.skipSpace();
            if (self.startsComment()) {
                self.skipComment();
                continue;
            }
            if (self.cursor == self.source.len) break;
            if (self.source[self.cursor] == '\n') {
                try self.finishLine();
                self.cursor += 1;
                self.at_line_start = true;
                continue;
            }
            if (self.source[self.cursor] == '+' and self.at_line_start) {
                self.cursor += 1;
                continue;
            }
            const raw = try self.readWord();
            defer self.allocator.free(raw.text);
            if (std.mem.eql(u8, raw.text, "+")) {
                try self.requireContinuation();
                continue;
            }
            try self.addWord(raw.text, raw.was_quoted);
            self.at_line_start = false;
        }
        try self.finishCommand();
        return self.script;
    }

    const RawWord = struct { text: []u8, was_quoted: bool };

    fn skipSpace(self: *Parser) void {
        while (self.cursor < self.source.len and (self.source[self.cursor] == ' ' or self.source[self.cursor] == '\t' or self.source[self.cursor] == '\r')) self.cursor += 1;
    }
    fn startsComment(self: *Parser) bool {
        return self.cursor + 1 < self.source.len and self.source[self.cursor] == '/' and self.source[self.cursor + 1] == '/';
    }
    fn skipComment(self: *Parser) void {
        while (self.cursor < self.source.len and self.source[self.cursor] != '\n') self.cursor += 1;
    }
    fn finishLine(self: *Parser) !void {
        try self.finishCommand();
    }
    fn finishCommand(self: *Parser) !void {
        if (self.command.words.items.len == 0) return;
        try self.script.commands.append(self.allocator, self.command);
        self.command = .{};
    }
    fn requireContinuation(self: *Parser) !void {
        self.skipSpace();
        if (self.startsComment()) self.skipComment();
        if (self.cursor >= self.source.len) return;
        if (self.source[self.cursor] != '\n') return error.UnexpectedNewline;
        self.cursor += 1;
        self.at_line_start = true;
    }

    fn readWord(self: *Parser) !RawWord {
        var out: std.ArrayList(u8) = .empty;
        errdefer out.deinit(self.allocator);
        var quoted = false;
        while (self.cursor < self.source.len) {
            const c = self.source[self.cursor];
            if (c == ' ' or c == '\t' or c == '\r' or c == '\n' or self.startsComment()) break;
            if (c == '\'') {
                quoted = true;
                try self.readQuoted(&out, '\'', false);
                continue;
            }
            if (c == '"') {
                quoted = true;
                try self.readQuoted(&out, '"', true);
                continue;
            }
            try out.append(self.allocator, c);
            self.cursor += 1;
        }
        if (out.items.len == 0 and !quoted) return error.InvalidErrorMarker;
        return .{ .text = try out.toOwnedSlice(self.allocator), .was_quoted = quoted };
    }

    fn readQuoted(self: *Parser, out: *std.ArrayList(u8), quote: u8, interpolation: bool) !void {
        self.cursor += 1;
        while (self.cursor < self.source.len) {
            const c = self.source[self.cursor];
            if (c == quote) {
                self.cursor += 1;
                return;
            }
            if (c == '\n') return error.UnterminatedQuote;
            if (interpolation and c == '\\') {
                self.cursor += 1;
                if (self.cursor == self.source.len) return error.InvalidEscape;
                const escaped = self.source[self.cursor];
                const value: u8 = switch (escaped) {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"', '\'', '\\', '$' => escaped,
                    else => return error.InvalidEscape,
                };
                try out.append(self.allocator, value);
                self.cursor += 1;
                continue;
            }
            if (interpolation and c == '$') {
                try self.expand(out);
                continue;
            }
            try out.append(self.allocator, c);
            self.cursor += 1;
        }
        return error.UnterminatedQuote;
    }

    fn expand(self: *Parser, out: *std.ArrayList(u8)) !void {
        self.cursor += 1;
        const start = self.cursor;
        if (self.cursor < self.source.len and self.source[self.cursor] == '{') {
            self.cursor += 1;
            const name_start = self.cursor;
            while (self.cursor < self.source.len and self.source[self.cursor] != '}') self.cursor += 1;
            if (self.cursor == self.source.len) return error.UnterminatedExpansion;
            if (self.cursor == name_start) return error.MissingExpansionName;
            try self.appendExpansion(out, self.source[name_start..self.cursor]);
            self.cursor += 1;
            return;
        }
        while (self.cursor < self.source.len and (std.ascii.isAlphanumeric(self.source[self.cursor]) or self.source[self.cursor] == '_')) self.cursor += 1;
        if (self.cursor == start) return error.MissingExpansionName;
        try self.appendExpansion(out, self.source[start..self.cursor]);
    }
    fn appendExpansion(self: *Parser, out: *std.ArrayList(u8), name: []const u8) !void {
        const value = if (self.options.lookup) |lookup| lookup(self.options.lookup_context, name) else null;
        if (value) |text| try out.appendSlice(self.allocator, text);
    }

    fn addWord(self: *Parser, text: []const u8, was_quoted: bool) !void {
        if (!was_quoted and std.mem.startsWith(u8, text, "--") and text.len == 3) return self.addFlag(&self.command.semantic_flags, text[2]);
        if (!was_quoted and text.len == 2 and text[0] == '-') return self.addFlag(&self.command.execution_flags, text[1]);
        if (!was_quoted and std.mem.eql(u8, text, "$")) {
            try self.command.words.append(self.allocator, .error_marker);
            return;
        }
        if (!was_quoted and std.mem.startsWith(u8, text, "$")) return error.InvalidErrorMarker;
        if (!was_quoted) if (try collection(text, self.allocator)) |ref| {
            errdefer self.allocator.free(ref.name);
            try self.command.words.append(self.allocator, .{ .collection = ref });
            return;
        };
        const owned = try self.allocator.dupe(u8, text);
        errdefer self.allocator.free(owned);
        try self.command.words.append(self.allocator, .{ .text = owned });
    }
    fn addFlag(self: *Parser, flags: *std.ArrayList(u8), flag: u8) !void {
        if (!std.ascii.isAlphabetic(flag)) return error.InvalidFlag;
        for (flags.items) |existing| if (existing == flag) return error.DuplicateFlag;
        try flags.append(self.allocator, flag);
    }
};

fn collection(text: []const u8, allocator: std.mem.Allocator) !?CollectionRef {
    const open = std.mem.lastIndexOfScalar(u8, text, '[') orelse return null;
    if (open == 0 or text.len < open + 3 or text[text.len - 1] != ']') return null;
    const index = std.fmt.parseUnsigned(usize, text[open + 1 .. text.len - 1], 10) catch return error.InvalidCollectionIndex;
    if (index == 0) return error.InvalidCollectionIndex;
    return .{ .name = try allocator.dupe(u8, text[0..open]), .index = index };
}

pub fn parse(allocator: std.mem.Allocator, source: []const u8, options: Options) !Script {
    var parser = Parser{ .allocator = allocator, .source = source, .options = options, .script = .{ .allocator = allocator } };
    return parser.run();
}

test "specified syntax produces typed commands" {
    var script = try parse(std.testing.allocator, "// CI generation\nci generate github --b +\n  -a\ntarget select targets[1]\nprint '$target' \"$name\"\n$\n", .{ .lookup = struct {
        fn get(_: ?*anyopaque, name: []const u8) ?[]const u8 {
            return if (std.mem.eql(u8, name, "name")) "rekindled" else null;
        }
    }.get });
    defer script.deinit();
    try std.testing.expectEqual(@as(usize, 4), script.commands.items.len);
    const ci = script.commands.items[0];
    try std.testing.expectEqualSlices(u8, "ci", ci.words.items[0].text);
    try std.testing.expectEqualSlices(u8, "b", ci.semantic_flags.items);
    try std.testing.expectEqualSlices(u8, "a", ci.execution_flags.items);
    try std.testing.expectEqual(@as(usize, 1), script.commands.items[1].words.items[2].collection.index);
    try std.testing.expectEqualSlices(u8, "$target", script.commands.items[2].words.items[1].text);
    try std.testing.expectEqualSlices(u8, "rekindled", script.commands.items[2].words.items[2].text);
    try std.testing.expect(script.commands.items[3].words.items[0] == .error_marker);
}

test "rejects invalid syntax rather than changing meaning" {
    try std.testing.expectError(error.UnterminatedQuote, parse(std.testing.allocator, "echo \"no", .{}));
    try std.testing.expectError(error.InvalidFlag, parse(std.testing.allocator, "ci --1", .{}));
    try std.testing.expectError(error.InvalidCollectionIndex, parse(std.testing.allocator, "target select targets[0]", .{}));
    try std.testing.expectError(error.UnexpectedNewline, parse(std.testing.allocator, "echo + nope", .{}));
}

test "all parser allocation failures release prior ownership" {
    try std.testing.checkAllAllocationFailures(std.testing.allocator, struct {
        fn run(allocator: std.mem.Allocator) !void {
            var script = try parse(
                allocator,
                "project setup sample --fabric\ntarget select targets[2]\nprint '$literal' \"$expanded\"\n",
                .{ .lookup = struct {
                    fn get(_: ?*anyopaque, name: []const u8) ?[]const u8 {
                        return if (std.mem.eql(u8, name, "expanded")) "value" else null;
                    }
                }.get },
            );
            defer script.deinit();
        }
    }.run, .{});
}
