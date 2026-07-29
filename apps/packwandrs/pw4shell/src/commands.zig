const std = @import("std");
const shell = @import("pw4shell");

pub const Error = error{
    UnknownCommand,
    MissingProjectName,
    ConflictingLoader,
    UnexpectedArguments,
};

pub const ProjectKind = enum { modpack, datapack, resourcepack, mod };
pub const Loader = enum { fabric, forge, neoforge, quilt };

pub const Request = union(enum) {
    project_setup: struct { kind: ProjectKind, name: []const u8, loader: ?Loader },
    ci_setup_forgejo,
    build: struct { project: ?[]const u8 },

    pub fn packwandArgv(self: Request, allocator: std.mem.Allocator) ![]const []const u8 {
        var out: std.ArrayList([]const u8) = .empty;
        switch (self) {
            .project_setup => |project| {
                try out.appendSlice(allocator, &.{ "packwand", "new", category(project.kind), project.name });
                if (project.loader) |loader| {
                    try out.appendSlice(allocator, &.{ "--loader", @tagName(loader) });
                }
            },
            .build => |build_request| {
                try out.appendSlice(allocator, &.{ "packwand", "build" });
                if (build_request.project) |project| {
                    try out.appendSlice(allocator, &.{ "--pack", project });
                }
            },
            .ci_setup_forgejo => return &.{},
        }
        return out.toOwnedSlice(allocator);
    }
};

pub fn resolve(command: *const shell.Command) !Request {
    const words = command.words.items;
    if (words.len >= 2 and is(words[0], "project") and is(words[1], "setup")) {
        return projectSetup(words[2..]);
    }
    if (words.len == 3 and is(words[0], "ci") and is(words[1], "setup") and is(words[2], "forgejo")) {
        return .ci_setup_forgejo;
    }
    if (words.len == 1 and is(words[0], "build")) {
        return .{ .build = .{ .project = null } };
    }
    if (words.len == 2 and is(words[0], "build")) {
        return .{ .build = .{ .project = text(words[1]) orelse return error.UnexpectedArguments } };
    }
    return error.UnknownCommand;
}

fn projectSetup(words: []const shell.Value) !Request {
    var kind: ProjectKind = .modpack;
    var loader: ?Loader = null;
    var name: ?[]const u8 = null;
    for (words) |word| {
        const value = text(word) orelse return error.UnexpectedArguments;
        if (kindFlag(value)) |flag_kind| {
            kind = flag_kind;
            continue;
        }
        if (loaderFlag(value)) |flag_loader| {
            if (loader != null and loader.? != flag_loader) return error.ConflictingLoader;
            loader = flag_loader;
            continue;
        }
        if (std.mem.startsWith(u8, value, "-")) return error.UnexpectedArguments;
        if (name != null) return error.UnexpectedArguments;
        name = value;
    }
    return .{ .project_setup = .{
        .kind = kind,
        .name = name orelse return error.MissingProjectName,
        .loader = loader,
    } };
}

fn kindFlag(value: []const u8) ?ProjectKind {
    if (std.mem.eql(u8, value, "-mp")) return .modpack;
    if (std.mem.eql(u8, value, "-dp")) return .datapack;
    if (std.mem.eql(u8, value, "-rp")) return .resourcepack;
    if (std.mem.eql(u8, value, "-md")) return .mod;
    return null;
}

fn loaderFlag(value: []const u8) ?Loader {
    if (std.mem.eql(u8, value, "--fabric")) return .fabric;
    if (std.mem.eql(u8, value, "--forge")) return .forge;
    if (std.mem.eql(u8, value, "--neoforge")) return .neoforge;
    if (std.mem.eql(u8, value, "--quilt")) return .quilt;
    return null;
}

fn category(kind: ProjectKind) []const u8 {
    return switch (kind) {
        .modpack => "modpacks",
        .datapack => "datapacks",
        .resourcepack => "resourcepacks",
        .mod => "mods",
    };
}

fn text(value: shell.Value) ?[]const u8 {
    return switch (value) {
        .text => |item| item,
        else => null,
    };
}

fn is(value: shell.Value, wanted: []const u8) bool {
    return if (text(value)) |item| std.mem.eql(u8, item, wanted) else false;
}

fn parsedRequest(source: []const u8) !struct { script: shell.Script, request: Request } {
    var script = try shell.parse(std.testing.allocator, source, .{});
    errdefer script.deinit();
    return .{ .request = try resolve(&script.commands.items[0]), .script = script };
}

test "project setup defaults to modpack and Packwand's default loader" {
    var parsed = try parsedRequest("project setup rekindled");
    defer parsed.script.deinit();
    const request = parsed.request.project_setup;
    try std.testing.expectEqual(ProjectKind.modpack, request.kind);
    try std.testing.expectEqual(@as(?Loader, null), request.loader);
}

test "project setup maps kind and loader flags" {
    var parsed = try parsedRequest("project setup rekindled -rp --fabric");
    defer parsed.script.deinit();
    const argv = try parsed.request.packwandArgv(std.testing.allocator);
    defer std.testing.allocator.free(argv);
    try std.testing.expectEqualStrings("resourcepacks", argv[2]);
    try std.testing.expectEqualStrings("fabric", argv[5]);
}

test "conflicting loaders fail before execution" {
    var script = try shell.parse(std.testing.allocator, "project setup demo --fabric --forge", .{});
    defer script.deinit();
    try std.testing.expectError(error.ConflictingLoader, resolve(&script.commands.items[0]));
}

test "build selects one project" {
    var parsed = try parsedRequest("build rekindled");
    defer parsed.script.deinit();
    const argv = try parsed.request.packwandArgv(std.testing.allocator);
    defer std.testing.allocator.free(argv);
    try std.testing.expectEqualStrings("--pack", argv[2]);
    try std.testing.expectEqualStrings("rekindled", argv[3]);
}

test "Forgejo setup is typed and has no child argv" {
    var parsed = try parsedRequest("ci setup forgejo");
    defer parsed.script.deinit();
    try std.testing.expect(parsed.request == .ci_setup_forgejo);
    try std.testing.expectEqual(@as(usize, 0), (try parsed.request.packwandArgv(std.testing.allocator)).len);
}
