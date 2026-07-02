package core

import (
	"os"
	"path/filepath"
	"runtime"

	"github.com/spf13/viper"
)

func GetPackwandLocalStore() (string, error) {
	if //goland:noinspection GoBoolExpressions
	runtime.GOOS == "linux" {
		// Prefer $XDG_DATA_HOME over $XDG_CACHE_HOME
		dataHome := os.Getenv("XDG_DATA_HOME")
		if dataHome != "" {
			return filepath.Join(dataHome, "packwand"), nil
		}
	}
	userConfigDir, err := os.UserConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(userConfigDir, "packwand"), nil
}

func GetPackwandLocalCache() (string, error) {
	userCacheDir, err := os.UserCacheDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(userCacheDir, "packwand"), nil
}

func GetPackwandInstallBinPath() (string, error) {
	localStore, err := GetPackwandLocalStore()
	if err != nil {
		return "", err
	}
	return filepath.Join(localStore, "bin"), nil
}

func GetPackwandInstallBinFile() (string, error) {
	binPath, err := GetPackwandInstallBinPath()
	if err != nil {
		return "", err
	}
	var exeName string
	if //goland:noinspection GoBoolExpressions
	runtime.GOOS == "windows" {
		exeName = "packwand.exe"
	} else {
		exeName = "packwand"
	}
	return filepath.Join(binPath, exeName), nil
}

func GetPackwandCache() (string, error) {
	configuredCache := viper.GetString("cache.directory")
	if configuredCache != "" {
		return configuredCache, nil
	}
	localStore, err := GetPackwandLocalCache()
	if err != nil {
		return "", err
	}
	return filepath.Join(localStore, "cache"), nil
}
