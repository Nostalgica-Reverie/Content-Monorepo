package curseforge

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/curseforge/murmur2"
	"github.com/spf13/cobra"
)

// TODO: make all of this less bad and hardcoded

// detectCmd represents the detect command
var detectCmd = &cobra.Command{
	Use:   "detect",
	Short: "Detect .jar files in the mods folder (experimental)",
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

		// Walk files in the mods folder
		var hashes []uint32
		modPaths := make(map[uint32]string)
		err = filepath.Walk("mods", func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if info.IsDir() {
				return nil
			}
			if !strings.HasSuffix(path, ".jar") && !strings.HasSuffix(path, ".litemod") {
				// TODO: make this less bad
				return nil
			}
			fmt.Println("Hashing " + path)
			bytes, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			hash := getByteArrayHash(bytes)
			hashes = append(hashes, hash)
			modPaths[hash] = path
			return nil
		})
		if err != nil {
			return err
		}
		fmt.Printf("Found %d files, submitting...\n", len(hashes))

		res, err := cfDefaultClient.getFingerprintInfo(hashes)
		if err != nil {
			return err
		}

		fmt.Printf("Successfully matched %d files\n", len(res.ExactFingerprints))
		if len(res.PartialMatches) > 0 {
			fmt.Println("The following fingerprints were partial and I don't know what to do!!!")
			for _, v := range res.PartialMatches {
				fmt.Printf("%s (%d)", modPaths[v], v)
			}
		}
		if len(res.UnmatchedFingerprints) > 0 {
			fmt.Printf("Failed to match the following %d files:\n", len(res.UnmatchedFingerprints))
			for _, v := range res.UnmatchedFingerprints {
				fmt.Printf("%s (%d)\n", modPaths[v], v)
			}
		}

		fmt.Println("Retrieving metadata...")
		ids := make([]uint32, len(res.ExactMatches))
		for i, v := range res.ExactMatches {
			ids[i] = v.ID
		}
		modInfos, err := cfDefaultClient.getModInfoMultiple(ids)
		if err != nil {
			return fmt.Errorf("failed to retrieve metadata: %w", err)
		}
		modInfosMap := make(map[uint32]modInfo)
		for _, v := range modInfos {
			modInfosMap[v.ID] = v
		}

		fmt.Println("Creating metadata files...")
		for _, v := range res.ExactMatches {
			err = createModFile(modInfosMap[v.ID], v.File, &index, false, "")
			if err != nil {
				return err
			}

			path, ok := modPaths[v.File.Fingerprint]
			if ok {
				err = os.Remove(path)
				if err != nil {
					return err
				}
			}
		}
		fmt.Println("Detection complete!")

		if _, err = index.Refresh(); err != nil {
			return err
		}
		return core.CommitChanges(&index, &pack)
	},
}

func init() {
	curseforgeCmd.AddCommand(detectCmd)
}

func getByteArrayHash(bytes []byte) uint32 {
	return murmur2.MurmurHash2(computeNormalizedArray(bytes), 1)
}

func computeNormalizedArray(bytes []byte) []byte {
	var newArray []byte
	for _, b := range bytes {
		if !isWhitespaceCharacter(b) {
			newArray = append(newArray, b)
		}
	}
	return newArray
}

func isWhitespaceCharacter(b byte) bool {
	return b == 9 || b == 10 || b == 13 || b == 32
}
