package curseforge

import (
	"archive/zip"
	"bufio"
	"bytes"
	"errors"
	"fmt"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/curseforge/packinterop"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func downloadCurseForgeImport(source string) (string, error) {
	response, err := http.Get(source) //nolint:gosec -- importing an explicitly supplied URL is the command's purpose.
	if err != nil {
		return "", err
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusOK {
		return "", fmt.Errorf("HTTP %d from %s", response.StatusCode, source)
	}
	tmp, err := os.CreateTemp("", "packwand-curseforge-import-*.zip")
	if err != nil {
		return "", err
	}
	path := tmp.Name()
	if _, err := io.Copy(tmp, response.Body); err != nil {
		tmp.Close()
		os.Remove(path)
		return "", err
	}
	if err := tmp.Close(); err != nil {
		os.Remove(path)
		return "", err
	}
	return path, nil
}

// normalizeImportPath produces a canonical form of a pack file path so that
// override files can be matched against metadata-referenced files even when
// the zip entry and the CurseForge API disagree on case or path separators.
// A trailing ".disabled" (used by the CurseForge launcher for disabled
// optional mods) is stripped so disabled override copies are matched too.
func normalizeImportPath(path string) string {
	if abs, err := filepath.Abs(path); err == nil {
		path = abs
	}
	path = strings.TrimSuffix(path, ".disabled")
	return strings.ToLower(filepath.ToSlash(path))
}

// importCmd represents the import command
var importCmd = &cobra.Command{
	Use:   "import [modpack path]",
	Short: "Import a curseforge modpack from a downloaded pack zip or an installed metadata json file",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		inputFile := args[0]
		var packImport packinterop.ImportPackMetadata

		// TODO: refactor/extract file checking?
		if strings.HasPrefix(inputFile, "http://") || strings.HasPrefix(inputFile, "https://") {
			downloaded, err := downloadCurseForgeImport(inputFile)
			if err != nil {
				fmt.Printf("Error downloading pack: %s\n", err)
				os.Exit(1)
			}
			defer os.Remove(downloaded)
			inputFile = downloaded
		}
		// Attempt to read from file
		var f *os.File
		inputFileStat, err := os.Stat(inputFile)
		if err == nil && inputFileStat.IsDir() {
			// Apparently os.Open doesn't fail when file given is a directory, only when it gets read
			err = errors.New("cannot open directory")
		}
		if err == nil {
			f, err = os.Open(inputFile)
		}
		if err != nil {
			found := false
			var errInstance error
			var errManifest error
			var errCurse error

			// Look for other files/folders
			if _, errInstance = os.Stat(filepath.Join(inputFile, "minecraftinstance.json")); errInstance == nil {
				inputFile = filepath.Join(inputFile, "minecraftinstance.json")
				found = true
			} else if _, errManifest = os.Stat(filepath.Join(inputFile, "manifest.json")); errManifest == nil {
				inputFile = filepath.Join(inputFile, "manifest.json")
				found = true
			} else if runtime.GOOS == "windows" {
				var dir string
				dir, errCurse = getCurseDir()
				if errCurse == nil {
					curseInstanceFile := filepath.Join(dir, "Minecraft", "Instances", inputFile, "minecraftinstance.json")
					if _, errCurse = os.Stat(curseInstanceFile); errCurse == nil {
						inputFile = curseInstanceFile
						found = true
					}
				}
			}

			if found {
				f, err = os.Open(inputFile)
				if err != nil {
					fmt.Printf("Error opening file: %s\n", err)
					os.Exit(1)
				}
			} else {
				fmt.Printf("Error opening file: %s\n", err)
				fmt.Printf("Also attempted minecraftinstance.json: %s\n", errInstance)
				fmt.Printf("Also attempted manifest.json: %s\n", errManifest)
				if errCurse != nil {
					fmt.Printf("Also attempted to load a Curse/Twitch modpack named \"%s\": %s\n", inputFile, errCurse)
				}
				os.Exit(1)
			}
		}
		defer f.Close()

		buf := bufio.NewReader(f)
		header, err := buf.Peek(2)
		if err != nil {
			fmt.Printf("Error reading file: %s\n", err)
			os.Exit(1)
		}

		// Check if file is a zip
		if string(header) == "PK" {
			// Read the whole file (as bufio doesn't work for zips)
			zipData, err := io.ReadAll(buf)
			if err != nil {
				fmt.Printf("Error reading file: %s\n", err)
				os.Exit(1)
			}
			// Get zip size
			stat, err := f.Stat()
			if err != nil {
				fmt.Printf("Error reading file: %s\n", err)
				os.Exit(1)
			}
			zr, err := zip.NewReader(bytes.NewReader(zipData), stat.Size())
			if err != nil {
				fmt.Printf("Error parsing zip: %s\n", err)
				os.Exit(1)
			}

			// Search the zip for minecraftinstance.json or manifest.json
			var metaFile *zip.File
			for _, v := range zr.File {
				if v.Name == "minecraftinstance.json" || v.Name == "manifest.json" {
					metaFile = v
				}
			}

			if metaFile == nil {
				fmt.Println("Can't find manifest.json or minecraftinstance.json, is this a valid pack?")
				os.Exit(1)
			}

			packImport = packinterop.ReadMetadata(packinterop.GetZipPackSource(metaFile, zr))
		} else {
			packImport = packinterop.ReadMetadata(packinterop.GetDiskPackSource(buf, filepath.ToSlash(filepath.Base(inputFile)), filepath.Dir(inputFile)))
		}

		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println("Failed to load existing pack, creating a new one...")

			// Create a new modpack
			indexFilePath := viper.GetString("init.index-file")
			_, err = os.Stat(indexFilePath)
			if os.IsNotExist(err) {
				// Create file
				err = os.WriteFile(indexFilePath, []byte{}, 0644)
				if err != nil {
					fmt.Printf("Error creating index file: %s\n", err)
					os.Exit(1)
				}
				fmt.Println(indexFilePath + " created!")
			} else if err != nil {
				fmt.Printf("Error checking index file: %s\n", err)
				os.Exit(1)
			}

			pack = core.Pack{
				Name:       packImport.Name(),
				Author:     packImport.PackAuthor(),
				Version:    packImport.PackVersion(),
				PackFormat: core.CurrentPackFormat,
				Index: struct {
					File       string `toml:"file"`
					HashFormat string `toml:"hash-format"`
					Hash       string `toml:"hash,omitempty"`
				}{
					File: indexFilePath,
				},
				Versions: packImport.Versions(),
			}
		} else {
			for component, version := range packImport.Versions() {
				packVersion, ok := pack.Versions[component]
				if !ok {
					fmt.Println("Set " + core.ComponentToFriendlyName(component) + " version to " + version)
				} else if packVersion != version {
					fmt.Println("Set " + core.ComponentToFriendlyName(component) + " version to " + version + " (previously " + packVersion + ")")
				}
				pack.Versions[component] = version
			}
		}
		index, err := pack.LoadIndex()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		modsList := packImport.Mods()
		modIDs := make([]uint32, len(modsList))
		for i, v := range modsList {
			modIDs[i] = v.ProjectID
		}

		fmt.Println("Querying Curse API for dependency info...")

		modInfos, err := cfDefaultClient.getModInfoMultiple(modIDs)
		if err != nil {
			fmt.Printf("Failed to obtain project information: %s\n", err)
			os.Exit(1)
		}

		modInfosMap := make(map[uint32]modInfo)
		for _, v := range modInfos {
			modInfosMap[v.ID] = v
		}

		// TODO: multithreading????

		modFileInfosMap := make(map[uint32]modFileInfo)
		referencedModPaths := make([]string, 0, len(modsList))
		successes := 0
		remainingFileIDs := make([]uint32, 0, len(modsList))

		// 1st pass: query mod metadata for every CurseForge file
		for _, v := range modsList {
			modInfoValue, ok := modInfosMap[v.ProjectID]
			if !ok {
				fmt.Printf("Failed to obtain information for project/file IDs %d/%d\n", v.ProjectID, v.FileID)
				continue
			}

			found := false
			var fileInfo modFileInfo
			for _, fileInfo = range modInfoValue.LatestFiles {
				if fileInfo.ID == v.FileID {
					found = true
					break
				}
			}
			if found {
				modFileInfosMap[v.FileID] = fileInfo
			} else {
				remainingFileIDs = append(remainingFileIDs, v.FileID)
			}
		}

		// 2nd pass: query files that weren't in the previous results
		fmt.Println("Querying Curse API for file info...")

		modFileInfos, err := cfDefaultClient.getFileInfoMultiple(remainingFileIDs)
		if err != nil {
			fmt.Printf("Failed to obtain project file information: %s\n", err)
			os.Exit(1)
		}

		for _, v := range modFileInfos {
			modFileInfosMap[v.ID] = v
		}

		// 3rd pass: create mod files for every file
		for _, v := range modsList {
			modInfoValue, ok := modInfosMap[v.ProjectID]
			if !ok {
				fmt.Printf("Failed to obtain project information for project/file IDs %d/%d\n", v.ProjectID, v.FileID)
				continue
			}

			modFileInfoValue, ok := modFileInfosMap[v.FileID]
			if !ok {
				fmt.Printf("Failed to obtain project file information for project/file IDs %d/%d\n", v.ProjectID, v.FileID)
				continue
			}

			err = createModFile(modInfoValue, modFileInfoValue, &index, v.OptionalDisabled, "")
			if err != nil {
				fmt.Printf("Failed to save project \"%s\": %s\n", modInfoValue.Name, err)
				os.Exit(1)
			}

			modFilePath := getPathForFile(modInfoValue.GameID, modInfoValue.ClassID, modInfoValue.PrimaryCategoryID, modInfoValue.Slug)
			referencedModPaths = append(referencedModPaths,
				normalizeImportPath(filepath.Join(filepath.Dir(modFilePath), modFileInfoValue.FileName)))

			fmt.Printf("Imported dependency \"%s\" successfully!\n", modInfoValue.Name)
			successes++
		}

		fmt.Printf("Successfully imported %d/%d dependencies!\n", successes, len(modsList))

		fmt.Println("Reading override files...")
		filesList, err := packImport.GetFiles()
		if err != nil {
			fmt.Printf("Failed to read override files: %s\n", err)
			os.Exit(1)
		}

		successes = 0
		for _, v := range filesList {
			filePath := index.ResolveIndexPath(v.Name())
			filePathNorm := normalizeImportPath(filePath)
			if slices.Contains(referencedModPaths, filePathNorm) {
				fmt.Printf("Ignored file \"%s\" (referenced by metadata)\n", filePath)
				successes++
				continue
			}
			if v.Name() == "manifest.json" || v.Name() == "minecraftinstance.json" || v.Name() == ".curseclient" {
				fmt.Printf("Ignored file \"%s\"\n", v.Name())
				successes++
				continue
			}

			f, err := os.Create(filePath)
			if err != nil {
				// Attempt to create the containing directory
				err2 := os.MkdirAll(filepath.Dir(filePath), os.ModePerm)
				if err2 == nil {
					f, err = os.Create(filePath)
				}
				if err != nil {
					fmt.Printf("Failed to write file \"%s\": %s\n", filePath, err)
					if err2 != nil {
						fmt.Printf("Failed to create directories: %s\n", err)
					}
					continue
				}
			}
			src, err := v.Open()
			if err != nil {
				fmt.Printf("Failed to read file \"%s\": %s\n", filePath, err)
				f.Close()
				continue
			}
			_, err = io.Copy(f, src)
			if err != nil {
				fmt.Printf("Failed to copy file \"%s\": %s\n", filePath, err)
				f.Close()
				src.Close()
				continue
			}

			fmt.Printf("Copied file \"%s\" successfully!\n", filePath)
			f.Close()
			src.Close()
			successes++
		}
		if len(filesList) > 0 {
			fmt.Printf("Successfully copied %d/%d files!\n", successes, len(filesList))
			if _, err = index.Refresh(); err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		} else {
			fmt.Println("No files copied!")
		}

		if err = core.CommitChanges(&index, &pack); err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	},
}

func init() {
	curseforgeCmd.AddCommand(importCmd)
}
