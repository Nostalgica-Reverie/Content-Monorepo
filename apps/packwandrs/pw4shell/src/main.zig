const std = @import("std");
const shell = @import("pw4shell");
const commands = @import("commands");

const forgejo_workflow =
    \\name: Packwand
    \\on:
    \\  push:
    \\  pull_request:
    \\  workflow_dispatch:
    \\jobs:
    \\  validate:
    \\    runs-on: ubuntu-latest
    \\    steps:
    \\      - uses: actions/checkout@v4
    \\      - uses: dtolnay/rust-toolchain@stable
    \\      - name: Build Packwand workspace
    \\        run: cargo build --workspace --locked
    \\      - name: Test Packwand workspace
    \\        run: cargo test --workspace --locked
    \\      - name: Validate Packwand projects
    \\        run: cargo run --locked -p packwand-cli -- doctor
    \\
;

pub fn main(init: std.process.Init) !void {
    const allocator = init.gpa;
    var args = try std.process.Args.Iterator.initAllocator(init.minimal.args, allocator);
    defer args.deinit();
    _ = args.next();

    var parts: std.ArrayList([]const u8) = .empty;
    while (args.next()) |arg| try parts.append(allocator, arg);
    defer parts.deinit(allocator);
    if (parts.items.len == 0) return error.MissingCommand;

    const source = if (parts.items.len == 1 and std.mem.endsWith(u8, parts.items[0], ".pw4"))
        try std.Io.Dir.cwd().readFileAlloc(init.io, parts.items[0], allocator, .limited(16 * 1024 * 1024))
    else
        try std.mem.join(allocator, " ", parts.items);
    defer allocator.free(source);
    var script = try shell.parse(allocator, source, .{});
    defer script.deinit();

    var stdout_buffer: [4096]u8 = undefined;
    var stderr_buffer: [4096]u8 = undefined;
    var stdout = std.Io.File.stdout().writer(init.io, &stdout_buffer);
    var stderr = std.Io.File.stderr().writer(init.io, &stderr_buffer);

    for (script.commands.items) |command| {
        const request = try commands.resolve(&command);
        switch (request) {
            .ci_setup_forgejo => {
                const cwd = std.Io.Dir.cwd();
                try cwd.createDirPath(init.io, ".forgejo/workflows");
                try cwd.writeFile(init.io, .{
                    .sub_path = ".forgejo/workflows/packwand.yml",
                    .data = forgejo_workflow,
                    .flags = .{ .exclusive = true },
                });
                try stdout.interface.writeAll("created .forgejo/workflows/packwand.yml\n");
            },
            else => {
                const argv = try request.packwandArgv(allocator);
                defer allocator.free(argv);
                const result = try std.process.run(allocator, init.io, .{
                    .argv = argv,
                    .stdout_limit = .limited(16 * 1024 * 1024),
                    .stderr_limit = .limited(16 * 1024 * 1024),
                    .expand_arg0 = .expand,
                    .create_no_window = false,
                });
                defer allocator.free(result.stdout);
                defer allocator.free(result.stderr);
                try stdout.interface.writeAll(result.stdout);
                try stderr.interface.writeAll(result.stderr);
                switch (result.term) {
                    .exited => |code| if (code != 0) return error.ChildFailed,
                    else => return error.ChildTerminated,
                }
            },
        }
    }
    try stdout.interface.flush();
    try stderr.interface.flush();
}
