package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"
	"github.com/spf13/cobra"
)

var cacheCmd = &cobra.Command{
	Use:   "cache",
	Short: "Inspect and maintain the shared download cache",
}

var cachePruneCmd = &cobra.Command{
	Use:   "prune",
	Short: "Remove cached download files no longer referenced by any pack",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		asJSON, _ := cmd.Flags().GetBool("json")

		cachePath, err := core.GetPackwandCache()
		if err != nil {
			llFail(err.Error())
		}
		index, err := core.LoadCacheIndexReadOnly(cachePath)
		if err != nil {
			llFail(fmt.Sprintf("failed to load cache index: %v", err))
		}

		referenced, err := collectReferencedHashes()
		if err != nil {
			llFail(fmt.Sprintf("failed to scan packs for referenced files (aborting to avoid pruning in-use cache entries): %v", err))
		}

		result, err := index.Prune(referenced, dryRun)
		if err != nil {
			llFail(fmt.Sprintf("prune failed: %v", err))
		}

		if asJSON {
			printJSON(result)
			return
		}

		verb := "removed"
		if dryRun {
			verb = "would remove"
		}
		fmt.Printf("%s %d/%d cache entries (%.1f MB)\n", verb, len(result.RemovedEntries), result.ScannedEntries, float64(result.RemovedBytes)/1e6)
	},
}

func init() {
	cachePruneCmd.Flags().Bool("dry-run", false, "List what would be removed without deleting anything")
	cachePruneCmd.Flags().Bool("json", false, "Output a JSON summary instead of plain text")
	cacheCmd.AddCommand(cachePruneCmd)
	cacheCmd.GroupID = GroupOther
	rootCmd.AddCommand(cacheCmd)
}

// collectReferencedHashes scans every pack subdir's index.toml and mod
// metafiles across the whole workspace, returning the set of lowercased
// download hashes mods currently depend on. Returns an error (rather than a
// partial set) if any pack subdir fails to parse, since pruning against an
// incomplete referenced set risks deleting files still in use.
func collectReferencedHashes() (map[string]struct{}, error) {
	root := workspace.ModpacksDir()
	targets, _ := workspace.CollectTargets(root, false, "", false)

	referenced := make(map[string]struct{})
	for _, dir := range targets {
		indexPath := filepath.Join(dir, "index.toml")
		if _, err := os.Stat(indexPath); err != nil {
			continue
		}
		idx, err := core.LoadIndex(indexPath)
		if err != nil {
			return nil, fmt.Errorf("%s: %w", dir, err)
		}
		mods, err := idx.LoadAllMods()
		if err != nil {
			return nil, fmt.Errorf("%s: %w", dir, err)
		}
		for _, m := range mods {
			if m.Download.Hash != "" {
				referenced[strings.ToLower(m.Download.Hash)] = struct{}{}
			}
		}
	}
	return referenced, nil
}
