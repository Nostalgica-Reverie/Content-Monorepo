const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const module = b.addModule("pw4shell", .{ .root_source_file = b.path("src/pw4shell.zig"), .target = target, .optimize = optimize });

    const commands = b.addModule("commands", .{ .root_source_file = b.path("src/commands.zig"), .target = target, .optimize = optimize });
    commands.addImport("pw4shell", module);

    const exe = b.addExecutable(.{ .name = "pw4shell", .root_module = b.createModule(.{ .root_source_file = b.path("src/main.zig"), .target = target, .optimize = optimize }) });
    exe.root_module.addImport("pw4shell", module);
    exe.root_module.addImport("commands", commands);
    b.installArtifact(exe);
    const run_cmd = b.addRunArtifact(exe);
    if (b.args) |args| run_cmd.addArgs(args);
    const run_step = b.step("run", "Run pw4shell");
    run_step.dependOn(&run_cmd.step);

    const tests = b.addTest(.{ .root_module = commands });
    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run pw4shell tests");
    test_step.dependOn(&run_tests.step);
}
