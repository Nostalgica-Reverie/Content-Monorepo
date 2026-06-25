package main

import (
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"time"
)

const servePort = "8080"

func cmdTest(args []string) {
	if len(args) < 1 {
		failUsage(verbUsage["test"])
	}
	packSubdir := absPath(args[0])

	if _, err := os.Stat(filepath.Join(packSubdir, "pack.toml")); err != nil {
		failNotFound(fmt.Sprintf("no pack.toml in %s", packSubdir))
	}
	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "install with 'go install github.com/packwiz/packwiz@latest' or point PACKWIZ_BIN at a binary")
	}
	if _, err := exec.LookPath("java"); err != nil {
		failEnv("java not found in PATH", "packwiz-installer is a Java jar; install a JRE/JDK")
	}
	installerJar := os.Getenv("PACKWIZ_INSTALLER_JAR")
	if installerJar == "" {
		failEnv("PACKWIZ_INSTALLER_JAR is not set", "download packwiz-installer-bootstrap.jar from https://github.com/packwiz/packwiz-installer-bootstrap/releases and export the path")
	}
	if _, err := os.Stat(installerJar); err != nil {
		failNotFound(fmt.Sprintf("packwiz-installer jar not found at %s", installerJar))
	}

	instanceDir := os.Getenv("SOMNUS_TEST_INSTANCE")
	if instanceDir == "" {
		instanceDir = "./.somnus-test-instance"
		fmt.Printf("SOMNUS_TEST_INSTANCE unset; using default %s\n", instanceDir)
	}
	if err := os.MkdirAll(instanceDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create instance dir %s: %v", instanceDir, err))
	}
	absInstance, err := filepath.Abs(instanceDir)
	if err != nil {
		fail(fmt.Sprintf("failed to resolve instance dir: %v", err))
	}
	absJar, err := filepath.Abs(installerJar)
	if err != nil {
		fail(fmt.Sprintf("failed to resolve jar path: %v", err))
	}

	fmt.Printf("starting packwiz serve in %s ...\n", packSubdir)
	serve := exec.Command(packwizBin(), "serve", "--port", servePort)
	serve.Dir = packSubdir
	serve.Stdout = os.Stderr
	serve.Stderr = os.Stderr
	if err := serve.Start(); err != nil {
		fail(fmt.Sprintf("failed to start packwiz serve: %v", err))
	}
	defer func() {
		if serve.Process != nil {
			_ = serve.Process.Kill()
			_, _ = serve.Process.Wait()
		}
	}()

	if !waitForPort("127.0.0.1:"+servePort, 10*time.Second) {
		fail("packwiz serve did not become ready on port " + servePort)
	}
	fmt.Println("packwiz serve is up.")

	packURL := fmt.Sprintf("http://localhost:%s/pack.toml", servePort)
	fmt.Printf("installing pack into %s ...\n", absInstance)
	installer := exec.Command("java", "-jar", absJar, "-g", packURL)
	installer.Dir = absInstance
	installer.Stdout = os.Stdout
	installer.Stderr = os.Stderr
	if err := installer.Run(); err != nil {
		fail(fmt.Sprintf("packwiz-installer failed: %v", err))
	}

	fmt.Printf("\ntest instance ready at %s\n", absInstance)
	fmt.Println("point your launcher (MultiMC/Prism) at it, or launch from there. (somnus does not launch the game.)")
}

func waitForPort(addr string, timeout time.Duration) bool {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		conn, err := net.DialTimeout("tcp", addr, 500*time.Millisecond)
		if err == nil {
			_ = conn.Close()
			return true
		}
		time.Sleep(200 * time.Millisecond)
	}
	return false
}
