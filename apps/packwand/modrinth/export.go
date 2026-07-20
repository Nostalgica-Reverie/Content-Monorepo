package modrinth

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"net/url"
	"os"
	"slices"
	"sort"
	"strconv"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmdshared"
	"github.com/spf13/viper"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
)

// exportCmd represents the export command
var exportCmd = &cobra.Command{
	Use:   "export",
	Short: "Export the current modpack into a .mrpack for Modrinth",
	Args:  cobra.NoArgs,
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("Loading modpack...")
		pack, err := core.LoadPack()
		if err != nil {
			return err
		}
		index, err := pack.LoadIndex()
		if err != nil {
			return err
		}
		// Do a refresh to ensure files are up to date
		if _, err = index.Refresh(); err != nil {
			return err
		}
		if err = index.Write(); err != nil {
			return err
		}

		fmt.Println("Reading external files...")
		mods, err := index.LoadAllMods()
		if err != nil {
			return fmt.Errorf("error reading file: %w", err)
		}

		fileName := viper.GetString("modrinth.export.output")
		if fileName == "" {
			fileName = pack.GetPackName() + ".mrpack"
		}
		expFile, err := os.Create(fileName)
		if err != nil {
			return fmt.Errorf("failed to create zip: %w", err)
		}
		exp := zip.NewWriter(expFile)

		// Add an overrides folder even if there are no files to go in it
		_, err = exp.Create("overrides/")
		if err != nil {
			return fmt.Errorf("failed to add overrides folder: %w", err)
		}

		fmt.Printf("Retrieving %v external files...\n", len(mods))

		restrictDomains := viper.GetBool("modrinth.export.restrictDomains")
		verify := viper.GetBool("modrinth.export.verify")

		for _, mod := range mods {
			if !canBeIncludedDirectly(mod, restrictDomains) {
				cmdshared.PrintDisclaimer(false)
				break
			}
		}

		// Fast path: mods whose metadata already carries every hash the
		// .mrpack manifest needs (sha1, sha512, size — persisted at
		// install/update time or backfilled by a previous export) go straight
		// into the manifest without touching the network or the cache.
		// --verify skips this and re-derives everything from file contents.
		manifestFiles := make([]PackFile, 0, len(mods))
		var sessionMods []*core.Mod
		directCount := 0
		for _, mod := range mods {
			if !verify && canBeIncludedDirectly(mod, restrictDomains) {
				if hashes, ok := mod.Download.ExportHashes(exportHashFormats); ok {
					entry, err := buildManifestEntry(mod, &index, hashes)
					if err != nil {
						return err
					}
					manifestFiles = append(manifestFiles, entry)
					directCount++
					continue
				}
			}
			sessionMods = append(sessionMods, mod)
		}
		if directCount > 0 {
			fmt.Printf("%d file(s) added to manifest from persisted hashes\n", directCount)
		}

		var backfilled []*core.Mod
		if len(sessionMods) > 0 {
			endMetaSpan := core.StartSpan("mr-export: metadata")
			session, err := core.CreateDownloadSession(sessionMods, exportHashFormats)
			endMetaSpan()
			if err != nil {
				return fmt.Errorf("error retrieving external files: %w", err)
			}

			cmdshared.ListManualDownloads(session)

			endDownloadSpan := core.StartSpan("mr-export: download+hash")
			for dl := range session.StartDownloads() {
				if canBeIncludedDirectly(dl.Mod, restrictDomains) {
					if dl.Error != nil {
						fmt.Printf("Download of %s (%s) failed: %v\n", dl.Mod.Name, dl.Mod.FileName, dl.Error)
						continue
					}
					for _, warning := range dl.Warnings {
						fmt.Printf("Warning for %s (%s): %v\n", dl.Mod.Name, dl.Mod.FileName, warning)
					}

					entry, err := buildManifestEntry(dl.Mod, &index, dl.Hashes)
					if err != nil {
						return err
					}
					manifestFiles = append(manifestFiles, entry)

					// Persist the hashes we just verified so the next export
					// can use the fast path for this mod.
					if backfillExportHashes(dl.Mod, dl.Hashes) {
						backfilled = append(backfilled, dl.Mod)
					}

					fmt.Printf("%s (%s) added to manifest\n", dl.Mod.Name, dl.Mod.FileName)
				} else {
					if dl.Mod.Side == core.ClientSide {
						_ = cmdshared.AddToZip(dl, exp, "client-overrides", &index)
					} else if dl.Mod.Side == core.ServerSide {
						_ = cmdshared.AddToZip(dl, exp, "server-overrides", &index)
					} else {
						_ = cmdshared.AddToZip(dl, exp, "overrides", &index)
					}
				}
			}
			endDownloadSpan()

			err = session.SaveIndex()
			if err != nil {
				return fmt.Errorf("error saving cache index: %w", err)
			}
		}

		if len(backfilled) > 0 {
			fmt.Printf("Persisting hashes for %d mod(s) to speed up future exports\n", len(backfilled))
			for _, m := range backfilled {
				format, hash, err := m.Write()
				if err != nil {
					fmt.Printf("Warning: failed to persist hashes for %s: %v\n", m.Name, err)
					continue
				}
				if err := index.RefreshFileWithHash(m.GetFilePath(), format, hash, true); err != nil {
					fmt.Printf("Warning: failed to refresh index for %s: %v\n", m.Name, err)
				}
			}
			if err := core.CommitChanges(&index, &pack); err != nil {
				fmt.Printf("Warning: failed to write index after persisting hashes: %v\n", err)
			}
		}

		// sort by `path` property before serialising to ensure reproducibility
		sort.Slice(manifestFiles, func(i, j int) bool {
			return manifestFiles[i].Path < manifestFiles[j].Path
		})

		dependencies := make(map[string]string)
		dependencies["minecraft"], err = pack.GetMCVersion()
		if err != nil {
			_ = exp.Close()
			_ = expFile.Close()
			return fmt.Errorf("error creating manifest: %w", err)
		}
		if quiltVersion, ok := pack.Versions["quilt"]; ok {
			dependencies["quilt-loader"] = quiltVersion
		} else if fabricVersion, ok := pack.Versions["fabric"]; ok {
			dependencies["fabric-loader"] = fabricVersion
		} else if forgeVersion, ok := pack.Versions["forge"]; ok {
			dependencies["forge"] = forgeVersion
		} else if neoforgeVersion, ok := pack.Versions["neoforge"]; ok {
			dependencies["neoforge"] = neoforgeVersion
		}

		manifest := Pack{
			FormatVersion: 1,
			Game:          "minecraft",
			VersionID:     pack.Version,
			Name:          pack.Name,
			Summary:       pack.Description,
			Files:         manifestFiles,
			Dependencies:  dependencies,
		}

		if len(pack.Version) == 0 {
			fmt.Println("Warning: pack.toml version field must not be empty to create a valid Modrinth pack")
		}

		manifestFile, err := exp.Create("modrinth.index.json")
		if err != nil {
			_ = exp.Close()
			_ = expFile.Close()
			return fmt.Errorf("error creating manifest: %w", err)
		}

		w := json.NewEncoder(manifestFile)
		w.SetIndent("", "    ") // Documentation uses 4 spaces
		err = w.Encode(manifest)
		if err != nil {
			_ = exp.Close()
			_ = expFile.Close()
			return fmt.Errorf("error writing manifest: %w", err)
		}

		cmdshared.AddNonMetafileOverrides(&index, exp)

		err = exp.Close()
		if err != nil {
			return fmt.Errorf("error writing export file: %w", err)
		}
		err = expFile.Close()
		if err != nil {
			return fmt.Errorf("error writing export file: %w", err)
		}

		fmt.Println("Modpack exported to " + fileName)
		return nil
	},
}

// exportHashFormats are the values the .mrpack manifest requires per file.
var exportHashFormats = []string{"sha1", "sha512", "length-bytes"}

// buildManifestEntry converts a mod plus its resolved hashes (persisted or
// freshly computed) into a .mrpack manifest entry.
func buildManifestEntry(mod *core.Mod, index *core.Index, hashes map[string]string) (PackFile, error) {
	path, err := index.RelIndexPath(mod.GetDestFilePath())
	if err != nil {
		return PackFile{}, fmt.Errorf("error resolving external file %s: %w", mod.Name, err)
	}

	fileSize, err := strconv.ParseUint(hashes["length-bytes"], 10, 64)
	if err != nil {
		return PackFile{}, fmt.Errorf("invalid length-bytes value for %s: %w", mod.Name, err)
	}

	// Create env options based on configured optional/side
	var envInstalled string
	if mod.Option != nil && mod.Option.Optional {
		envInstalled = "optional"
	} else {
		envInstalled = "required"
	}
	var clientEnv, serverEnv string
	switch mod.Side {
	case core.ClientSide:
		clientEnv, serverEnv = envInstalled, "unsupported"
	case core.ServerSide:
		clientEnv, serverEnv = "unsupported", envInstalled
	default: // UniversalSide / EmptySide
		clientEnv, serverEnv = envInstalled, envInstalled
	}

	// Modrinth URLs must be RFC3986
	u, err := core.ReencodeURL(mod.Download.URL)
	if err != nil {
		fmt.Printf("Error re-encoding download URL: %s\n", err.Error())
		u = mod.Download.URL
	}

	return PackFile{
		Path:   path,
		Hashes: map[string]string{"sha1": hashes["sha1"], "sha512": hashes["sha512"]},
		Env: &struct {
			Client string `json:"client"`
			Server string `json:"server"`
		}{Client: clientEnv, Server: serverEnv},
		Downloads: []string{u},
		FileSize:  uint32(fileSize),
	}, nil
}

// backfillExportHashes copies hashes computed during an export into the mod's
// persisted download section (extra-hashes + size). Returns true when the mod
// changed and needs its metadata file rewritten.
func backfillExportHashes(mod *core.Mod, hashes map[string]string) bool {
	changed := false
	for _, format := range []string{"sha1", "sha512"} {
		value := hashes[format]
		if value == "" || format == mod.Download.HashFormat {
			continue
		}
		if mod.Download.ExtraHashes[format] != value {
			if mod.Download.ExtraHashes == nil {
				mod.Download.ExtraHashes = make(map[string]string)
			}
			mod.Download.ExtraHashes[format] = value
			changed = true
		}
	}
	if lb := hashes["length-bytes"]; lb != "" {
		if size, err := strconv.ParseUint(lb, 10, 64); err == nil && size != 0 && mod.Download.Size != size {
			mod.Download.Size = size
			changed = true
		}
	}
	return changed
}

var whitelistedHosts = []string{
	"cdn.modrinth.com",
	"github.com",
	"raw.githubusercontent.com",
	"gitlab.com",
}

func canBeIncludedDirectly(mod *core.Mod, restrictDomains bool) bool {
	if mod.Download.Mode == core.ModeURL || mod.Download.Mode == "" {
		if !restrictDomains {
			return true
		}

		modUrl, err := url.Parse(mod.Download.URL)
		if err == nil {
			if slices.Contains(whitelistedHosts, modUrl.Host) {
				return true
			}
		}
	}
	return false
}

func init() {
	modrinthCmd.AddCommand(exportCmd)
	exportCmd.Flags().Bool("restrictDomains", true, "Restricts domains to those allowed by modrinth.com")
	exportCmd.Flags().StringP("output", "o", "", "The file to export the modpack to")
	exportCmd.Flags().Bool("verify", false, "Ignore persisted hashes and re-download/re-hash every file (slow; use to audit persisted metadata)")
	_ = viper.BindPFlag("modrinth.export.restrictDomains", exportCmd.Flags().Lookup("restrictDomains"))
	_ = viper.BindPFlag("modrinth.export.output", exportCmd.Flags().Lookup("output"))
	_ = viper.BindPFlag("modrinth.export.verify", exportCmd.Flags().Lookup("verify"))
}
