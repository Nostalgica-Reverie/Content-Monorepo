package cmd

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/clistyle"
	"github.com/spf13/cobra"
)

func init() {
	llJSONMinifyCmd.Flags().Bool("check", false, "Report files that would shrink and exit 1 instead of rewriting them")
	llJSONMinifyCmd.Flags().Bool("strict", false, "Fail on files that are not valid JSON instead of skipping them")
	llJSONCmd.AddCommand(llJSONMinifyCmd)

	llJSONCmd.GroupID = GroupBuildExport
	rootCmd.AddCommand(llJSONCmd)
}

var llJSONCmd = &cobra.Command{
	Use:   "json",
	Short: "JSON utilities for pack files",
}

var llJSONMinifyCmd = &cobra.Command{
	Use:   "minify <path...>",
	Short: "Minify .json/.mcmeta files in place (recurses into directories); invalid JSON is skipped with a warning",
	Long: "Strips insignificant whitespace from JSON files so built and published artifacts ship smaller. " +
		"Directories are walked recursively for .json and .mcmeta files; .git and node_modules are never " +
		"entered. Files that do not parse as strict JSON (e.g. JSON5/commented configs) are skipped with a " +
		"warning unless --strict is set. Key order and number formatting are preserved.",
	Args: cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		check, _ := cmd.Flags().GetBool("check")
		strict, _ := cmd.Flags().GetBool("strict")

		res, err := minifyJSONPaths(args, check, strict)
		if err != nil {
			llFail(err.Error())
		}
		verb := "minified"
		if check {
			verb = "would minify"
		}
		fmt.Printf(clistyle.IconOK+" %s %d of %d JSON file(s), saving %s\n",
			verb, res.minified, res.scanned, formatByteSize(res.saved))
		if res.skipped > 0 {
			llWarn("skipped %d file(s) that did not parse as strict JSON", res.skipped)
		}
		if check && res.minified > 0 {
			os.Exit(1)
		}
	},
}

type minifyResult struct {
	scanned  int
	minified int
	skipped  int
	saved    int64
}

// minifyJSONPaths minifies every .json/.mcmeta file under the given files or
// directories. With check set it only counts; with strict set an unparsable
// file is an error rather than a skip.
func minifyJSONPaths(paths []string, check, strict bool) (minifyResult, error) {
	var res minifyResult
	for _, root := range paths {
		info, err := os.Stat(root)
		if err != nil {
			return res, fmt.Errorf("cannot stat %s: %v", root, err)
		}
		if !info.IsDir() {
			if err := minifyJSONFile(root, check, strict, &res); err != nil {
				return res, err
			}
			continue
		}
		err = filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if d.IsDir() {
				name := d.Name()
				if name == ".git" || name == "node_modules" {
					return filepath.SkipDir
				}
				return nil
			}
			if !isJSONFilename(d.Name()) {
				return nil
			}
			return minifyJSONFile(path, check, strict, &res)
		})
		if err != nil {
			return res, err
		}
	}
	return res, nil
}

func isJSONFilename(name string) bool {
	return strings.HasSuffix(name, ".json") || strings.HasSuffix(name, ".mcmeta")
}

func minifyJSONFile(path string, check, strict bool, res *minifyResult) error {
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("cannot read %s: %v", path, err)
	}
	res.scanned++

	var buf bytes.Buffer
	if err := json.Compact(&buf, data); err != nil {
		if strict {
			return fmt.Errorf("%s is not valid JSON: %v", path, err)
		}
		llWarn("skipping %s: not valid JSON (%v)", path, err)
		res.skipped++
		return nil
	}
	if buf.Len() >= len(data) {
		return nil
	}

	res.minified++
	res.saved += int64(len(data) - buf.Len())
	if check {
		fmt.Printf("  would minify %s (%s -> %s)\n", path, formatByteSize(int64(len(data))), formatByteSize(int64(buf.Len())))
		return nil
	}
	mode := os.FileMode(0o644)
	if info, err := os.Stat(path); err == nil {
		mode = info.Mode().Perm()
	}
	if err := os.WriteFile(path, buf.Bytes(), mode); err != nil {
		return fmt.Errorf("cannot write %s: %v", path, err)
	}
	return nil
}

func formatByteSize(n int64) string {
	switch {
	case n >= 1<<20:
		return fmt.Sprintf("%.1f MiB", float64(n)/(1<<20))
	case n >= 1<<10:
		return fmt.Sprintf("%.1f KiB", float64(n)/(1<<10))
	default:
		return fmt.Sprintf("%d B", n)
	}
}
