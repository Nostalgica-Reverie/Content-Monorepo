package cmd

import (
	"fmt"
	"os/exec"
	"path/filepath"
	"sort"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// The nix command family replaces the standalone packwiz2nix generator: it
// uses packwand's own pack/index/mod parsing and download cache to produce
// the checksums.json consumed by lib/packwiz2nix's mkPackwizPackages, so
// hashes come from verified downloads rather than eval-time fetches.

var nixCmd = &cobra.Command{
	Use:   "nix",
	Short: "Nix integration (packwiz2nix-compatible outputs)",
}

var nixGenCmd = &cobra.Command{
	Use:   "gen",
	Short: "Generate a packwiz2nix checksums.json for this pack (or --all packs)",
	Long: `Generates the checksums.json consumed by lib/packwiz2nix (mkPackwizPackages,
mkModLinks, mkMultiMCPack) from the currently loaded pack. Mod files are
resolved through packwand's download cache, so already-fetched files are
hashed for free and new files are downloaded and verified against their
metadata hashes first.

Only URL-mode mods in the mods/ folder are included; CurseForge metadata-mode
files have no static URL and are skipped with a warning.`,
	Args: cobra.NoArgs,
	Run: func(c *cobra.Command, args []string) {
		if viper.GetBool("nix.gen.all") {
			runNixGenAll()
			return
		}
		if err := runNixGen(viper.GetString("nix.gen.output")); err != nil {
			cmdshared.Fail(err.Error())
		}
	},
}

func init() {
	nixCmd.AddCommand(nixGenCmd)
	AddToGroup(nixCmd, GroupOther)

	nixGenCmd.Flags().String("output", "checksums.json", "Path to write the checksums file to, relative to the pack directory")
	_ = viper.BindPFlag("nix.gen.output", nixGenCmd.Flags().Lookup("output"))
	nixGenCmd.Flags().Bool("all", false, "Generate for every pack subdir in the workspace (run from the repo root)")
	_ = viper.BindPFlag("nix.gen.all", nixGenCmd.Flags().Lookup("all"))
}

type nixChecksumEntry struct {
	URL    string `json:"url"`
	Sha256 string `json:"sha256"`
}

func runNixGen(output string) error {
	pack, err := core.LoadPack()
	if err != nil {
		return err
	}
	index, err := pack.LoadIndex()
	if err != nil {
		return err
	}
	mods, err := index.LoadAllMods()
	if err != nil {
		return err
	}

	// packwiz2nix's format covers the mods/ folder: keys are .pw.toml
	// basenames, values are the download URL and sha256 of the file.
	var eligible []*core.Mod
	for _, mod := range mods {
		inModsFolder := filepath.Base(filepath.Dir(mod.GetFilePath())) == "mods"
		if !inModsFolder {
			continue
		}
		if mod.Download.Mode != "" && mod.Download.Mode != core.ModeURL {
			Warn("%s: %s downloads have no static URL; skipped (Nix cannot fetch it)", mod.Name, mod.Download.Mode)
			continue
		}
		if mod.Download.URL == "" {
			Warn("%s: no download URL; skipped", mod.Name)
			continue
		}
		eligible = append(eligible, mod)
	}
	if len(eligible) == 0 {
		return fmt.Errorf("no URL-mode mods found in the mods/ folder")
	}

	checksums := make(map[string]nixChecksumEntry, len(eligible))
	err = cmdshared.WithSpinner(fmt.Sprintf("Resolving sha256 hashes for %d mod(s)", len(eligible)), func(update func(string)) error {
		session, err := core.CreateDownloadSession(eligible, []string{"sha256"})
		if err != nil {
			return fmt.Errorf("failed to create download session: %w", err)
		}
		resolved := 0
		for dl := range session.StartDownloads() {
			if dl.Error != nil {
				return fmt.Errorf("%s: %w", dl.Mod.Name, dl.Error)
			}
			for _, warning := range dl.Warnings {
				Warn("%s: %v", dl.Mod.Name, warning)
			}
			sha256, ok := dl.Hashes["sha256"]
			if !ok {
				_ = dl.File.Close()
				return fmt.Errorf("%s: no sha256 obtained", dl.Mod.Name)
			}
			_ = dl.File.Close()
			checksums[filepath.Base(dl.Mod.GetFilePath())] = nixChecksumEntry{
				URL:    dl.Mod.Download.URL,
				Sha256: sha256,
			}
			resolved++
			update(fmt.Sprintf("%d/%d  %s", resolved, len(eligible), dl.Mod.Name))
		}
		return session.SaveIndex()
	})
	if err != nil {
		return err
	}

	packDir := filepath.Dir(viper.GetString("pack-file"))
	outPath := output
	if !filepath.IsAbs(outPath) {
		outPath = filepath.Join(packDir, output)
	}
	if err := workspace.WriteJSON(outPath, checksums); err != nil {
		return err
	}
	fmt.Println(Success("wrote " + outPath + Dim(fmt.Sprintf("  (%d mod(s))", len(checksums)))))
	return nil
}

// runNixGenAll generates checksums for every eligible pack subdir in the
// workspace by re-invoking packwand in each (mirroring workspace update).
func runNixGenAll() {
	Chdir()
	targets, _ := workspace.CollectTargets(workspace.ModpacksDir(), false, "", false)
	if len(targets) == 0 {
		fmt.Println("no pack subdirs found")
		return
	}
	sort.Strings(targets)
	Header(fmt.Sprintf("Generating Nix checksums for %d pack subdir(s), up to %d in parallel", len(targets), workspace.MaxConcurrent()))
	sched := workspace.NewScheduler(workspace.MaxConcurrent())
	slots := workspace.CacheSlotCount()
	dones := make([]<-chan error, len(targets))
	for i, dir := range targets {
		dones[i] = sched.Submit(workspace.Task{
			Name: dir,
			Needs: []workspace.Resource{
				workspace.Resource("subdir:" + dir),
				workspace.CacheSlot(dir, slots),
			},
			Run: func() error {
				c := exec.Command(workspace.SelfBin(), "nix", "gen")
				c.Dir = dir
				workspace.ConfigureSubprocess(c)
				return workspace.StreamCommand(c, dir)
			},
		})
	}
	sched.Close()
	failed := 0
	for i, done := range dones {
		if err := <-done; err != nil {
			Warn("%s: nix gen failed: %v", targets[i], err)
			failed++
		}
	}
	if failed > 0 {
		cmdshared.Failf("%d of %d pack(s) failed", failed, len(targets))
	}
	fmt.Println(Success(fmt.Sprintf("all %d pack(s) generated", len(targets))))
}
